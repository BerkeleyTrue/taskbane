# taskbane
a mobile first taskwarrior ui

## TODO:

- [ ] add create task view

## Done

- [x] add single task view with annotations
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
- [x] create sqlx task storage
  - [x] fix: utf decoding issue
  - [x] add tests suite
- [x] add complete task action
- [x] fix: mark done in detail view
