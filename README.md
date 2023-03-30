# Carrier Log Discord Bot
Carrier Log Bot for Discord for Elite Dangerous. <br>
Give it a bot token and a channel as well as the path to the journal logs, and it will write departures from the carrier into the discord channel.

## Setup
<a href=https://www.rust-lang.org/tools/install>Cargo</a> is of course required, and you need to set up your <a href=https://discord.com/developers/applications>own Discord bot</a>

First of all clone the repo:

```git clone https://github.com/arne-fuchs/carrier_log_bot.git```

Go into the repo and edit the .env file and add the missing parameters.

```cd carrier_log_bot && nano .env```

* BOT_TOKEN: The token of your bot. You'll find it if you have set up your <a href=https://discord.com/developers/applications>own Discord bot</a> under the 'Bot' tab.
* CHANNEL_ID: The id of the channel, where the bot should write into. Please be aware that the bot needs access to the channel and that the bot is invited to the server. You can get the channel id by right-clicking on the channel and press 'Copy ID'.
* JOURNAL_PATH: The path where your journals are located. For Linux, it is usually ```~/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous``` but it usually needs the absolut path. For Windows it is ```%USERPROFILE%\\Saved Games\\Frontier Developments\\Elite Dangerous```. Don't know the rules there.

It is extremely recommended to use the <a href=https://github.com/rfvgyhn/min-ed-launcher>min-ed-launcher</a> to launch the bot with Elite, or otherwise you always have to do that manually
Go into following file after running elite once with the min-ed-launcher:
```nano ~/.config/min-ed-launcher/settings.json```

Add the bot as process (here a linux example):
```
"processes": [
	{
      	  "fileName": "PATH_TO_GIT_REPO/carrier_log_bot/start.sh",
      	  "arguments": ""
	}
    ],
```

Now it should start along with Elite. If it works the bot should be online in discord. Please note that the start scripts waits 60 seconds, so you can log in and that the journal log for the session is being created when the bot starts. Otherwise, it will choose an old log and false information will be printed into the channel.