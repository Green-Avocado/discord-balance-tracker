# discord-balance-tracker

[![Rust](https://github.com/Green-Avocado/discord-balance-tracker/actions/workflows/rust.yml/badge.svg)](https://github.com/Green-Avocado/discord-balance-tracker/actions/workflows/rust.yml)
[![Docker Image CI](https://github.com/Green-Avocado/discord-balance-tracker/actions/workflows/docker-image.yml/badge.svg)](https://github.com/Green-Avocado/discord-balance-tracker/actions/workflows/docker-image.yml)

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

amount:String - the amount in dollars to owe

description:String - description of the transaction

user:User - the user to owe to

### `/bill amount description [user0 ... user9]`

amount:String - the amount in dollars to owe

description:String - description of the transaction

[user0 ... user9]:User - the users to bill
