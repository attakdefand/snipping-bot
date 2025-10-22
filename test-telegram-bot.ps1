# Test Telegram Bot Script
# This script tests if your Telegram bot is working correctly

# Your bot token (replace with your actual token if different)
$botToken = "7997864014:AAHwrL5g6AX4mBd38RAJjT6hUMvbapzdIes"

# Your chat ID (replace with your actual chat ID if different)
$chatId = "385968548"

# Test message
$message = "🚀 Test message from snipping-bot repository! Telegram notifications are working correctly."

# Send the message
try {
    $uri = "https://api.telegram.org/bot$botToken/sendMessage"
    $body = @{
        chat_id = $chatId
        text = $message
    }
    
    $response = Invoke-RestMethod -Uri $uri -Method Post -Body $body
    
    if ($response.ok -eq $true) {
        Write-Host "✅ Success! Message sent to Telegram." -ForegroundColor Green
        Write-Host "Message ID: $($response.result.message_id)"
    } else {
        Write-Host "❌ Failed to send message. Response: $($response | ConvertTo-Json)" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Error sending message: $($_.Exception.Message)" -ForegroundColor Red
}