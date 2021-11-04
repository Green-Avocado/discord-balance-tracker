# discord-balance-tracker

Discord bot for managing group finances

## Running:

```
$ DISCORD_TOKEN={TOKEN}\
 APPLICATION_ID={ID}\
 cargo run
```

OR

`.env`:

```
DISCORD_TOKEN={TOKEN}
APPLICATION_ID={ID}
```

```
$ cargo run
```

## Commands

### `/balance`

No parameters

### `/owe amount description user`

amount:Integer - the amount in cents to owe

description:String - description of the transaction

user:User - the user to owe to

### `/bill amount description [user0 ... user9]`

amount:Integer - the amount in cents to bill to each user

description:String - description of the transaction

[user0 ... user9]:User - the users to bill
