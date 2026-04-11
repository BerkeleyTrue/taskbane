# taskbane

a mobile first taskwarrior ui. Point it to a sync server, login with passkey auth, authorize users against the taskdb. 

## Deployment

### Environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `DB_URL` | yes | — | SQLite connection string (e.g. `sqlite:///var/lib/taskbane/taskbane.db`) |
| `TASK_URL` | yes | — | Taskchampion sync server URL |
| `TASK_CLIENT_ID` | yes | — | Taskchampion client UUID |
| `TASK_SECRET` | yes | — | Taskchampion encryption secret |
| `ORIGIN` | yes | — | WebAuthn origin (e.g. `https://tasks.example.com`) |
| `RP_ID` | no | `localhost` | WebAuthn relying party ID (e.g. `tasks.example.com`) |
| `RP_NAME` | no | `taskbane` | WebAuthn relying party display name |
| `PORT` | no | `3000` | HTTP port to listen on |
| `PUBLIC_DIR` | no | `public` | Path to static assets directory. Set to `$out/share/taskbane/public` when running from a Nix build. |

### NixOS

Add the flake as an input and configure a systemd service:

```nix
# flake.nix
inputs.taskbane.url = "github:youruser/taskbane";

# nixos configuration
{ inputs, pkgs, ... }:
let
  taskbane = inputs.taskbane.packages.x86_64-linux.default;
in {
  systemd.services.taskbane = {
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${taskbane}/bin/taskbane";
      WorkingDirectory = "${taskbane}/share/taskbane";
      EnvironmentFile = "/var/lib/taskbane/.env";
      DynamicUser = true;
      StateDirectory = "taskbane";
    };
  };
}
```

## TODO:

- [ ] Move replica to mutex

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
- [x] add create task view
  - [x] parse dates 
  - [x] add deps typeahead
  - [x] post endpoint
  - [x] redirect on success
- [x] loading indicator
- [x] catch all error handler
- [x] annotate task
- [x] add secondary passkeys
