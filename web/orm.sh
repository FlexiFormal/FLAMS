sea-orm-cli migrate -u sqlite:/home/jazzpirate/.immt/users.sqlite?mode=rwc
sea-orm-cli generate entity -u sqlite:/home/jazzpirate/.immt/users.sqlite?mode=rwc -o orm/src/entities