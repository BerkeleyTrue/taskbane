# taskbane
a mobile first taskwarrior ui

## TODO:

- [x] create sqlx task storage
  - [ ] fix: utf decoding issue
  - [ ] add tests suite
- [ ] add complete task action
- [ ] add create task view
- [ ] add single task view with annotations

## Done

- [x] authorize use through task and key
  - [x] prevent unauthorised user from seeing tasks
  - [x] unauthorised users should be shown authorize page with task token
    - [x] show user authorize button, search for token
    - [x] copy to clipboard
    - [x] find token in tasks
    - [x] verify token
    - [x] authorize user
- [x] add session flash
  - [x] create template
  - [x] send down for errors
  - [x] add to session storage
