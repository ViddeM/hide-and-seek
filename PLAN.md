# Hide and Seek game

This project is intended to be a mobile-first website utility for people playing hide-and-seek, a game based on the gameshow by Jetlagged the game on youtube & nebula.

The idea is that there is a number of teams consisting of 1 or more players each.
On a rotating basis the teams take turn "hiding" somewhere on the map whilst the other teams (the seekers) collaborate in trying to find them.
The seekers do this by asking questions to the hiders (via external communication tools such as WhatsApp or Discord) which usually involve the current position of the hider and the seekers. GPS tracking is also managed externally by the players for now.
After being asked a question the hiders gets to draw & pick a number of cards (depending on the question) from a deck, these cards can either be bonuses for the hiders (such as extra time) or curses they can play to slow down the seekers.

## The map
During the setup of the game, a map should be created (these should also be savable such that they can be reused for future games).
A map covers a specific region of the globe, it comes in three sizes (small/medium/large) that affect the game rules, what questions are allowed etc.
Depending on the map, valid hiding places are e.g. train stations, tram stops or maybe airports.
All valid maps should have a strict set of valid stops, as well as clearly defined answers to questions, e.g. administrative regions, what valid amusement parks are available etc.

## The app
This app should act as a utility for both seekers and hiders, with each role having a separate view.
The roles have strict view isolation: seekers cannot see the hiders' view and vice versa. This is enforced server-side.

### Seeker view
The main seeker view is a map of the playable area where seekers can mark areas as "excluded" based on answers to questions received over external communication.
Seekers can add questions available for the given map size, e.g. "Are you within 5km of X?", and based on the hider's answer mark the corresponding area as excluded with a darker shade or similar.
If the answer to "within 5km" is "yes", the area outside that circle is excluded. If "no", the inside is excluded.
The seeker view should update in real-time across all connected seeker devices using WebSockets, so all seekers see exclusion zones as teammates add them.

### Hider view
The hider view shows the relevant game information for the hiding team, including the card drawing mechanic when responding to questions.
The hider view is strictly isolated from the seeker view — hiders must not be able to access or infer what the seekers have marked.

## Authentication
The app uses a game-code-based authentication system (similar to Jackbox or Kahoot), with no persistent user accounts required:
- A host creates a game and receives a short alphanumeric game code.
- Players join by entering the game code and are assigned to a team with a role (hider or seeker).
- The server issues a signed JWT encoding the game ID, team ID, and role.
- Every request and WebSocket message is validated against this token.
- Role-based access control is enforced server-side: hider tokens cannot access seeker endpoints and vice versa.

## Units
The app should use SI units for all measurements.

## Tech stack
Fullstack Rust project utilizing Dioxus.
The frontend should prioritize a web target with a particular focus on mobile design (as it will mainly be played on mobile devices), however, the UI should be implemented to be as generic (e.g. in a common module) as possible to allow for adding e.g. a native mobile target in the future.
The Rust code should follow best practices, avoid panics, unwraps etc. as far as possible and utilize the type system to disallow invalid states/behaviours as much as possible.

### API
The backend exposes a REST API for standard CRUD operations and uses WebSockets for real-time updates (exclusion zones, game state changes).

### Database
Any data storage necessary should utilize a PostgreSQL database, using the `sqlx` Rust crate.

## Startup
Parameters to the API should be set using the `clap` crate, however, they should (as far as possible) also support the ENV feature so that they can be set using environment variables.
All environment variables should be provided with reasonable default/example values in a `.env.example` file.
To support `.env` files the `dotenvy` crate should be used.
During startup, all configuration values should be validated as far as possible and if any of them is invalid, the app should crash before starting up.
The configs should then be stored in a `Config` struct (or similar) that is available during runtime so they are only read once during startup and can never be used without having been validated first.

## Logging
The project should utilize logging using the `env_logger` crate and `log` crate so that it is easy to follow what is happening on the backend (API) portion of the project.
We should log, at a minimum, the following:
 1) Any request received, whether it is valid or not.
 2) If it is valid, the main parameters for the request (if possible all, but only if it's not too long — e.g. we should not log a 1KB JSON request body).
 3) The outcome of the request (response code and simplified outcome).
 4) Any errors encountered using WARN level logs if it is due to a malformed request or similar, and ERROR level logs if there is something that has gone wrong internally on the server side.

## Resources
- Game store: https://store.nebula.tv/collections/jetlag/products/hideandseek
- Community wiki: https://jetlag.fandom.com/wiki/Hide_%2B_Seek
