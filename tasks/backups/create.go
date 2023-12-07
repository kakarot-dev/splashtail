package backups

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"slices"
	"splashtail/state"
	"splashtail/tasks"
	"splashtail/utils"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

func init() {
	tasks.RegisterTask(&ServerBackupCreateTask{})
}

func countMap(m map[string]int) int {
	var count int

	for _, v := range m {
		count += v
	}

	return count
}

// Backs up messages of a channel
//
// Note that attachments are only backed up if withAttachments is true and f.Size() < fileSizeWarningThreshold
//
// Note that this function does not write the messages to the file, it only returns them
func backupChannelMessages(logger *zap.Logger, f *iblfile.AutoEncryptedFile, channelID string, allocation int, withAttachments bool) ([]*BackupMessage, error) {
	var finalMsgs []*BackupMessage
	var currentId string
	for {
		// Fetch messages
		if allocation < len(finalMsgs) {
			// We've gone over, break
			break
		}

		limit := min(100, allocation-len(finalMsgs))

		messages, err := state.Discord.ChannelMessages(channelID, limit, "", currentId, "")

		if err != nil {
			return nil, fmt.Errorf("error fetching messages: %w", err)
		}

		for _, msg := range messages {
			im := BackupMessage{
				Message: msg,
			}

			if withAttachments && f.Size() < fileSizeWarningThreshold {
				am, bufs, err := createAttachmentBlob(logger, msg)

				if err != nil {
					return nil, fmt.Errorf("error creating attachment blob: %w", err)
				}

				im.AttachmentMetadata = am
				im.attachments = bufs
			}

			finalMsgs = append(finalMsgs, &im)
		}

		if len(messages) < limit {
			// We've reached the end
			break
		}
	}

	return finalMsgs, nil
}

func createAttachmentBlob(logger *zap.Logger, msg *discordgo.Message) ([]AttachmentMetadata, map[string]*bytes.Buffer, error) {
	var attachments []AttachmentMetadata
	var bufs = map[string]*bytes.Buffer{}
	for _, attachment := range msg.Attachments {
		if attachment.Size > maxAttachmentFileSize {
			attachments = append(attachments, AttachmentMetadata{
				ID:          attachment.ID,
				Name:        attachment.Filename,
				URL:         attachment.URL,
				ProxyURL:    attachment.ProxyURL,
				Size:        attachment.Size,
				ContentType: attachment.ContentType,
				Errors:      []string{"Attachment is too large to be saved."},
			})
			continue
		}

		// Download the attachment
		var url string

		if attachment.ProxyURL != "" {
			url = attachment.ProxyURL
		} else {
			url = attachment.URL
		}

		resp, err := http.Get(url)

		if err != nil {
			logger.Error("Error downloading attachment", zap.Error(err), zap.String("url", url))
			return attachments, nil, fmt.Errorf("error downloading attachment: %w", err)
		}

		bt, err := io.ReadAll(resp.Body)

		if err != nil {
			logger.Error("Error reading attachment", zap.Error(err), zap.String("url", url))
			return attachments, nil, fmt.Errorf("error reading attachment: %w", err)
		}

		bufs[attachment.ID] = bytes.NewBuffer(bt)

		attachments = append(attachments, AttachmentMetadata{
			ID:     attachment.ID,
			Name:   attachment.Filename,
			Errors: []string{},
		})
	}

	return attachments, bufs, nil
}

// A task to create backup a server
type ServerBackupCreateTask struct {
	// The ID of the task
	TaskID string `json:"task_id"`

	// The ID of the server
	ServerID string `json:"server_id"`

	// Backup options
	BackupOpts BackupOpts `json:"backup_opts"`
}

// $SecureStorage/guilds/$guildId/backups/$taskId
func (t *ServerBackupCreateTask) dir() string {
	return fmt.Sprintf("%s/guilds/%s/backups/%s", state.Config.Meta.SecureStorage, t.ServerID, t.TaskID)
}

// $SecureStorage/guilds/$guildId/backups/$taskId/backup.arbackup
func (t *ServerBackupCreateTask) path() string {
	return t.dir() + "/backup.arbackup"
}

func (t *ServerBackupCreateTask) Validate() error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	return nil
}

func (t *ServerBackupCreateTask) Exec(l *zap.Logger, tx pgx.Tx) error {
	l.Info("Beginning backup")

	if t.BackupOpts.MaxMessages == 0 {
		t.BackupOpts.MaxMessages = totalMaxMessages
	}

	if t.BackupOpts.PerChannel == 0 {
		t.BackupOpts.PerChannel = defaultPerChannel
	}

	if t.BackupOpts.PerChannel < minPerChannel {
		return fmt.Errorf("per_channel cannot be less than %d", minPerChannel)
	}

	if t.BackupOpts.MaxMessages > totalMaxMessages {
		return fmt.Errorf("max_messages cannot be greater than %d", totalMaxMessages)
	}

	if t.BackupOpts.PerChannel > t.BackupOpts.MaxMessages {
		return fmt.Errorf("per_channel cannot be greater than max_messages")
	}

	if t.BackupOpts.BackupAttachments && !t.BackupOpts.BackupMessages {
		return fmt.Errorf("cannot backup attachments without messages")
	}

	if len(t.BackupOpts.SpecialAllocations) == 0 {
		t.BackupOpts.SpecialAllocations = make(map[string]int)
	}

	f, err := iblfile.NewAutoEncryptedFile("")

	if err != nil {
		return fmt.Errorf("error creating file: %w", err)
	}

	err = f.WriteJsonSection(t.BackupOpts, "backup_opts")

	if err != nil {
		return fmt.Errorf("error writing backup options: %w", err)
	}

	// Fetch the bots member object in the guild
	l.Info("Fetching bots current state in server")
	m, err := state.Discord.GuildMember(t.ServerID, state.BotUser.ID)

	if err != nil {
		return fmt.Errorf("error fetching bots member object: %w", err)
	}

	err = f.WriteJsonSection(m, "debug/bot") // Write bot member object to debug section

	if err != nil {
		return fmt.Errorf("error writing bot member object: %w", err)
	}

	l.Info("Backing up server settings")

	// Fetch guild
	g, err := state.Discord.Guild(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching guild: %w", err)
	}

	// With servers now backed up, get the base permissions now
	basePerms := utils.BasePermissions(g, m)

	// Write base permissions to debug section
	err = f.WriteJsonSection(basePerms, "debug/base_permissions")

	if err != nil {
		return fmt.Errorf("error writing base permissions: %w", err)
	}

	l.Info("Backing up guild channels")

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching channels: %w", err)
	}

	g.Channels = channels

	cb := CoreBackup{
		Guild: g,
	}

	if len(g.Roles) == 0 {
		l.Info("Backing up guild roles", zap.String("taskId", t.TaskID))

		// Fetch roles of guild
		roles, err := state.Discord.GuildRoles(t.ServerID)

		if err != nil {
			return fmt.Errorf("error fetching roles: %w", err)
		}

		g.Roles = roles
	}

	if len(g.Stickers) == 0 {
		l.Info("Backing up guild stickers", zap.String("taskId", t.TaskID))

		// Fetch stickers of guild
		stickers, err := state.Discord.Request("GET", discordgo.EndpointGuildStickers(t.ServerID), nil)

		if err != nil {
			return fmt.Errorf("error fetching stickers: %w", err)
		}

		var s []*discordgo.Sticker

		err = json.Unmarshal(stickers, &s)

		if err != nil {
			return fmt.Errorf("error unmarshalling stickers: %w", err)
		}

		g.Stickers = s
	}

	err = f.WriteJsonSection(cb, "core")

	if err != nil {
		return fmt.Errorf("error writing core backup: %w", err)
	}

	// Backup messages
	if t.BackupOpts.BackupMessages {
		l.Info("Calculating message backup allocations", zap.String("taskId", t.TaskID))

		// Create channel map to allow for easy channel lookup
		var channelMap map[string]*discordgo.Channel = make(map[string]*discordgo.Channel)

		for _, channel := range channels {
			channelMap[channel.ID] = channel
		}

		// Allocations per channel
		var perChannelBackupMap = make(map[string]int)

		// First handle the special allocations
		for channelID, allocation := range t.BackupOpts.SpecialAllocations {
			if c, ok := channelMap[channelID]; ok {
				// Error on bad channels for special allocations
				if !slices.Contains(allowedChannelTypes, c.Type) {
					return fmt.Errorf("special allocation channel %s is not a valid channel type", c.ID)
				}

				perms := utils.MemberChannelPerms(basePerms, g, m, c)

				if perms&discordgo.PermissionViewChannel != discordgo.PermissionViewChannel {
					return fmt.Errorf("special allocation channel %s is not readable by the bot", c.ID)
				}

				if countMap(perChannelBackupMap) >= t.BackupOpts.MaxMessages {
					continue
				}

				perChannelBackupMap[channelID] = allocation
			}
		}

		for _, channel := range channels {
			// Discard bad channels
			if !slices.Contains(allowedChannelTypes, channel.Type) {
				continue
			}

			perms := utils.MemberChannelPerms(basePerms, g, m, channel)

			if perms&discordgo.PermissionViewChannel != discordgo.PermissionViewChannel {
				continue
			}

			if countMap(perChannelBackupMap) >= t.BackupOpts.MaxMessages {
				perChannelBackupMap[channel.ID] = 0 // We still need to include the channel in the allocations
			}

			if _, ok := perChannelBackupMap[channel.ID]; !ok {
				perChannelBackupMap[channel.ID] = t.BackupOpts.PerChannel
			}
		}

		l.Info("Created channel backup allocations", zap.Any("alloc", perChannelBackupMap), zap.Strings("botDisplayIgnore", []string{"alloc"}))

		// Backup messages
		for channelID, allocation := range perChannelBackupMap {
			if allocation == 0 {
				continue
			}

			l.Info("Backing up channel messages", zap.String("channelId", channelID))

			var leftovers int

			msgs, err := backupChannelMessages(l, f, channelID, allocation, t.BackupOpts.BackupAttachments)

			if err != nil {
				if t.BackupOpts.IgnoreMessageBackupErrors {
					l.Error("error backing up channel messages", zap.Error(err))
					leftovers = allocation
				} else {
					return fmt.Errorf("error backing up channel messages: %w", err)
				}
			} else {
				if len(msgs) < allocation {
					leftovers = allocation - len(msgs)
				}

				// Write messages of this section
				f.WriteJsonSection(msgs, "messages/"+channelID)

				for _, msg := range msgs {
					if len(msg.attachments) > 0 {
						for id, buf := range msg.attachments {
							f.WriteSection(buf, "attachments/"+id)
						}
					}
				}
			}

			if leftovers > 0 && t.BackupOpts.RolloverLeftovers {
				// Find a new channel with 0 allocation
				for channelID, allocation := range perChannelBackupMap {
					if allocation == 0 {
						msgs, err := backupChannelMessages(l, f, channelID, leftovers, t.BackupOpts.BackupAttachments)

						if err != nil {
							if t.BackupOpts.IgnoreMessageBackupErrors {
								l.Error("error backing up channel messages [leftovers]", zap.Error(err))
								continue // Try again
							} else {
								return fmt.Errorf("error backing up channel messages [leftovers]: %w", err)
							}
						} else {
							f.WriteJsonSection(msgs, "messages/"+channelID)

							for _, msg := range msgs {
								if len(msg.attachments) > 0 {
									for id, buf := range msg.attachments {
										f.WriteSection(buf, "attachments/"+id)
									}
								}
							}
							break
						}
					}
				}
			}
		}
	}

	metadata := iblfile.Meta{
		CreatedAt:      time.Now(),
		Protocol:       iblfile.Protocol,
		Type:           fileType,
		EncryptionData: f.EncDataMap,
	}

	ifmt, err := iblfile.GetFormat(fileType)

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return fmt.Errorf("error getting format: %w", err)
	}

	metadata.FormatVersion = ifmt.Version

	err = f.WriteJsonSection(metadata, "meta")

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return fmt.Errorf("error writing metadata: %w", err)
	}

	// Create dir
	err = os.MkdirAll(t.dir(), 0700)

	if err != nil {
		l.Error("Failed to create directory", zap.Error(err))
		return fmt.Errorf("error creating directory: %w", err)
	}

	// Write backup to path
	file, err := os.Create(t.path())

	if err != nil {
		l.Error("Failed to create file", zap.Error(err))
		return fmt.Errorf("error creating file: %w", err)
	}

	defer file.Close()

	err = f.WriteOutput(file)

	if err != nil {
		l.Error("Failed to write backup", zap.Error(err))
		return fmt.Errorf("error writing backup: %w", err)
	}

	l.Info("Successfully created backup")

	return nil
}

func (t *ServerBackupCreateTask) Info() *tasks.TaskInfo {
	return &tasks.TaskInfo{
		TaskID:     t.TaskID,
		Name:       "create_backup",
		For:        tasks.Pointer("g/" + t.ServerID),
		TaskFields: t,
		Expiry:     1 * time.Hour,
	}
}

func (t *ServerBackupCreateTask) Output() *tasks.TaskOutput {
	return &tasks.TaskOutput{
		Path: t.path(),
	}
}

func (t *ServerBackupCreateTask) Set(set *tasks.TaskSet) tasks.Task {
	t.TaskID = set.TaskID

	return t
}
