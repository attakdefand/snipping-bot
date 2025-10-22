# Telegram Secrets Verification Guide

This guide will help you verify that the Telegram secrets are properly configured in your GitHub repository.

## Common Issues and Solutions

### 1. Missing Secrets
The most common issue is that the required secrets are not configured in your GitHub repository.

**Required Secrets:**
- `TELEGRAM_BOT_TOKEN`: Your Telegram bot token
- `TELEGRAM_CHAT_ID`: Your Telegram chat ID

### 2. Incorrect Secret Values
Make sure the secret values are correct:
- The bot token should be in the format: `123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ`
- The chat ID should be a numeric value (e.g., `123456789`)

### 3. Repository Permissions
Make sure you have the necessary permissions to access the repository secrets.

## How to Verify Secrets Configuration

1. Go to your GitHub repository: https://github.com/attakdefand/snipping-bot
2. Click on "Settings" tab
3. In the left sidebar, click "Secrets and variables" → "Actions"
4. Verify that both `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` are present

## How to Add Missing Secrets

If the secrets are missing:

1. Click "New repository secret"
2. Add the `TELEGRAM_BOT_TOKEN` secret:
   - Name: `TELEGRAM_BOT_TOKEN`
   - Value: `7997864014:AAHwrL5g6AX4mBd38RAJjT6hUMvbapzdIes`
3. Click "Add secret"
4. Click "New repository secret" again
5. Add the `TELEGRAM_CHAT_ID` secret:
   - Name: `TELEGRAM_CHAT_ID`
   - Value: `385968548`
6. Click "Add secret"

## Testing the Configuration

After adding the secrets, you can test the configuration by:

1. Going to the "Actions" tab in your repository
2. Selecting the "Test Telegram Notification" workflow
3. Clicking "Run workflow" and selecting the "test-telegram" branch

## Troubleshooting Tips

1. Make sure you've sent at least one message to your bot in Telegram
2. Verify that the bot token is still valid
3. Check that the chat ID is correct by sending another message and using the getUpdates API
4. If you're still having issues, try creating a new bot and updating the secrets