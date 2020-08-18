const Discord = require('discord.js');
const bot = new Discord.Client();
const fs = require('fs');
const cheerio = require('cheerio');
const request = require('request');
const himalaya = require('himalaya');


// Reading settings
const settings = JSON.parse(fs.readFileSync('settings.json', {encoding: 'utf8'}));
const language = JSON.parse(fs.readFileSync(settings.lang, {encoding: 'utf8'}));
const date = new Date();

/**
 * Storing competition data
 */
class Competition {
    /**
     * Creates competition based on the parameters
     * 
     * @param {*Date} date - date of the competition
     * @param {*string} location - location of the competition
     * @param {*string} organiser - organiser of the competition
     * @param {*type} organiser - competition type
     */
    constructor(date, location, organiser, type) {
        this.date = date;
        this.location = location;
        this.organiser = organiser;
        this.type = type;
    }
}

// Competitions are stored here
var competitionsDP = {
    "list": [],
    "expiresAt": 0
};
var competitionsVL = {
    "list": [],
    "expiresAt": 0
};

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
                // Cache
                if(competitionsDP.expiresAt > date) {
                    commandParser(msg, args, competitionsDP.list);
                    return;
                }
                request({
                    method: 'GET',
                    // Luckily KŠP has an API :)
                    url: "https://isksp.pzs.si/Dogodek/Dogodek_GetDataForGrid"
                }, (err, res, body) => {
                    if (err)
                        return console.error(err);

                    const competitions = JSON.parse(body).Data;
                    // Sorting the data by ids
                    competitions.sort((a, b) => parseInt(b.Id) - parseInt(a.Id));

                    // Removing old competitions
                    for(let i = 0; i < competitions.length; i++) {
                        if(competitions[i].Sezona < date.getFullYear()) {
                            competitions.splice(i, competitions.length - i);
                            break;
                        }

                        // Adding competitions to the list
                        competitionsDP.list.push(new Competition(
                            new Date(parseInt(competitions[i].DatumZacetka.replace(/[^0-9]/g, ""))),
                            competitions[i].Kraj,
                            competitions[i].Klub,
                            competitions[i].TipNaziv
                        ));
                        // Expiration after 1 day
                        competitionsDP.expiresAt = new Date().setDate(date.getDate() + 1);
                    }
                    competitionsDP.list.reverse();
                    commandParser(msg, args, competitionsDP.list);
                });
                break;
            case "vl":
                // Cache
                if(competitionsVL.expiresAt > date) {
                    commandParser(msg, args, competitionsVL.list);
                    return;
                }
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
                        let competition = $(competitions[i]).text();
                        year = competition.split(".");
                        
                        if(!year[2].startsWith(date.getFullYear())) {
                            competitions.splice(i, competitions.length - i);
                            break;
                        }

                        competition = competition.split(",");

                        // Creating date of competition from string
                        let competitionDate = new Date(competition[0].split(".").reverse());
                        // Adding competitions to the list
                        competitionsVL.list.push(new Competition(
                            competitionDate,
                            competition[1].replace(" ", "")
                        ));
                        // Expiration after 1 day
                        competitionsVL.expiresAt = new Date().setDate(date.getDate() + 1);
                    }
                    competitionsVL.list.reverse();
                    commandParser(msg, args, competitionsVL.list)
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

async function commandParser(msg, args, competitionList) {
    const number = args[1];
    if(number == null) {
        msg.channel.send(new Discord.MessageEmbed().setTitle(language.competition.thisYear + " - " + args[0].toUpperCase()).setColor('#FFA600'));
        for(let i = 0; i < competitionList.length; i++) {
            msg.channel.send(getDateAndLocationMessage(competitionList, i));
        }
    }
    else if(number == "naslednja" || number == 0) {
        var message;
        for(var i = 0; i < competitionList.length; i++) {            
            // Checking whether this match has not been organised yet
            if(competitionList[i].date >= date) {
                message = getDateAndLocationMessage(competitionList, i);

                msg.channel.send(new Discord.MessageEmbed()
                    .setTitle(language.competition.next + " - " + args[0].toUpperCase())
                    .setDescription(
                        // 86400000 is equal to one day
                        language.competition.timeUntillNext.replace(
                            "${days}", Math.ceil((competitionList[i].date.getTime() - date.getTime()) / 86400000)
                        )
                    )
                    .setColor('#EBE700'));
                
                break;
            }
        }
        // If message is null, it means that there are no more competitions this year
        if(!message)
            message = language.competition.noMore;
        
        msg.channel.send(message)
        //console.log(new Date(competitionList[i].date - date));
        //msg.channel.send(new Date(competitionList[i].date - date).getMonth());
    }
    else {
        msg.channel.send(getDateAndLocationMessage(competitionList, number - 1));
    }
}

/**
 * Gets the message for the competition,
 * styled in "DD.MM.YYYY, location".
 * 
 * @param {array} competitionList 
 * @param {integer} index 
 */
function getDateAndLocationMessage(competitionList, index) {
    let dateFormat = competitionList[index].date.toLocaleDateString('en-US', { year: "numeric", month: "short", day: "numeric" });
    return dateFormat + ", " + competitionList[index].location;
}
