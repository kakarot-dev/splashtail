TEST__USER_ID := 728871946456137770
CDN_PATH := /failuremgmt/cdn/antiraid


stcore:
	CGO_ENABLED=0 go build -v 
reloadwebserver:
	systemctl restart splashtail-staging-webserver
updatebot:
	make buildbot && cp -v botv2/target/release/botv2 botv2
restartwebserver:
	make stcore
	make buildbot
	systemctl stop splashtail-staging-webserver
	cp -v botv2/target/release/botv2 botv2
	systemctl start splashtail-staging-webserver
reloadjobserver:
	systemctl restart splashtail-staging-jobs
all:
	make buildbot && make buildmewldwebui && make stcore 
buildbot:
	cd botv2 && cargo build --release
buildmewldwebui:
	cd mewld_web/ui && npm i && npm run build && cd ../
tests:
	CGO_ENABLED=0 go test -v -coverprofile=coverage.out ./...
ts:
	rm -rvf $(CDN_PATH)/dev/bindings/splashtail
	~/go/bin/tygo generate

	# Copy over go types
	mkdir -p $(CDN_PATH)/dev/bindings/splashtail/go
	mkdir -p bot/src/generatedTypes
	cp -rf types $(CDN_PATH)/dev/bindings/splashtail/go

	# Patch to change package name to 'splashtail_types'
	sed -i 's:package types:package splashtail_types:g' $(CDN_PATH)/dev/bindings/splashtail/go/types/{go/*,*.ts}
	cp -rf $(CDN_PATH)/dev/bindings/splashtail/* website/src/lib/generated
	rm -rf website/src/lib/generated/go

promoteprod:
	rm -rf ../prod2
	cd .. && cp -rf staging prod2
	echo "prod" > ../prod2/config/current-env
	cd ../prod2 && make && rm -rf ../prod && mv -vf ../prod2 ../prod && systemctl restart splashtail-prod
	cd ../prod && make ts

	# Git push to "current-prod" branch
	cd ../prod && git branch current-prod && git add -v . && git commit -m "Promote staging to prod" && git push -u origin HEAD:current-prod --force
