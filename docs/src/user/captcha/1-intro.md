# Introduction

*WARNING: Captcha's are being heavily churned/rewritten into a new Lua plugin*

A CAPTCHA is a security measure that can be used to try and filter out the more basic and spammy bots from accessing your server. It presents a challenge that users must complete to prove they are human. This can help protect your server from unwanted automated access and ensure that real users can interact with your community. **Note that CAPTCHA's are NOT foolproof and that more sophistiated bots may still bypass them.**

Of course, with the current landscape with AI everywhere, just serving a single CAPTCHA type (or config) does not really work. Furthermore, AntiRaid recognizes that different servers have different needs. Therefore, we provide a flexible CAPTCHA system, fully integrated with our Lua Templating system that allows you to completely customize the CAPTCHA experience for your users. This occurs through filters.
