const Discord = require('discord.js');
const bot = new Discord.Client();
const fs = require('fs');
const cheerio = require('cheerio');
const request = require('request');
const himalaya = require('himalaya');
const fetch = require('node-fetch');
const http = require('http');


// Reading settings
const settings = JSON.parse(fs.readFileSync('settings.json', {encoding: 'utf8'}));
const language = JSON.parse(fs.readFileSync(settings.lang, {encoding: 'utf8'}));
 

bot.on('ready', () => {
    console.log('Booted up!');
    bot.user.setPresence({
        status: 'online',
        activity: {
            name: settings.activity.name,
            type: 'PLAYING'
        }
    })
});

 
bot.on('message', msg => {
    if (msg.content.startsWith(settings.command.start)) {
        // Bot command was used
        const args = msg.content.split(" ");
        args.shift(); // removing first part of the command
        
        if(args.length == 0) {
            msg.channel.send(language.arguments.missing);
            return;
        }
        switch(args[0].toLowerCase()) {
            case "dp":
                msg.channel.send("Tekma dp ... ne vem še kdaj je");
                break;
            case "vl":
                //let message = getVLData(args.shift());
                //console.log("Message: " + message);
                //msg.channel.send(message);
                const number = args[1];
                request({
                    method: 'GET',
                    url: "https://climbers.si/index.asp?Page=ArhivVzhodneLige_Tekme"
                }, (err, res, body) => {
                    if (err)
                        return console.error(err);

                    let $ = cheerio.load(body);
                    let competitions = $('select#izbTekme option').toArray();

                    // Removing old competitions
                    for(let i = 0; i < competitions.length; i++) {
                        let date = new Date().getFullYear();
                        let competition = $(competitions[i]).text();
                        competition = competition.split(".");
                        
                        if(!competition[2].startsWith(date)) {
                            competitions.splice(i, competitions.length - i);
                            break;
                        }
                    }
                    competitions.reverse();

                    if(number == null) {
                        msg.channel.send(language.competition.thisYear);
                        for(let i = 0; i < competitions.length; i++) {
                            msg.channel.send($(competitions[i]).text());
                        }
                        return;
                    }
                    else if(number == "naslednja")
                        msg.channel.send($(competitions[0]).text());
                    else
                        msg.channel.send($(competitions[number - 1]).text());
                });
                break;
            default:
                msg.channel.send(language.arguments.invalid.concat(args[0]))
                break;
        }
    }
});


// Logging in
let token = settings.token;
if(!token)
    token = process.env.BOT_TOKEN;
bot.login(token);
