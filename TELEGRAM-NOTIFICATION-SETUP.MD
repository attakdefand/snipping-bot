# Telegram Notification Setup for GitHub Actions

This document explains how to set up Telegram notifications for GitHub Actions in the snipping-bot repository.

## Prerequisites

1. You must have a Telegram bot token (already provided)
2. You must have your Telegram chat ID (retrieved from the bot)

## Setting up Telegram Secrets

1. Go to your GitHub repository at https://github.com/attakdefand/snipping-bot
2. Click on "Settings" tab
3. In the left sidebar, click "Secrets and variables" → "Actions"
4. Click "New repository secret" and add these two secrets:

### Secret 1: TELEGRAM_BOT_TOKEN
- Name: `TELEGRAM_BOT_TOKEN`
- Value: `7997864014:AAHwrL5g6AX4mD38RAJjT6hUMvbapzdIes`

### Secret 2: TELEGRAM_CHAT_ID
- Name: `TELEGRAM_CHAT_ID`
- Value: `385968548`

## How to Get Your Chat ID

If you need to retrieve your chat ID again:

1. Open Telegram and search for your bot
2. Send any message to your bot (like "Hello")
3. Run this command in your terminal:
   ```bash
   curl https://api.telegram.org/bot[YOUR_BOT_TOKEN]/getUpdates
   ```
4. Look for the "chat" → "id" field in the response

## Testing the Notification

After setting up the secrets, any new push to the repository will trigger a Telegram notification with:
- Repository information
- Branch name
- Commit details
- Author information
- Job status for all CI pipeline steps
- Link to view detailed results

## Troubleshooting

If you're not receiving notifications:

1. Check that both secrets are correctly added to your repository
2. Verify that the bot token is valid
3. Ensure you've sent at least one message to the bot (required for the chat ID to be active)
4. Check the GitHub Actions logs for any error messages