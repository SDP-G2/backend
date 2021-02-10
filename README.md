# SDP Backend

## Overview
This repository contains all of the backend systems for the SPD project. Mainly this is the system that allows communication between the frontend, whether this is a mobile app or webapp, and the robot.

Adding on, the backend repo is where the server for Claynce should be run in.

# Instructions
1. Install Docker AND Docker-Compose
2. Open terminal (Mac/Linux) or powershell (Windows)
3. Change directory to `sdp-backend`
4. Type in console: `docker build -t sdp`
5. Change directory back to the parent folder `backend`
6. Type in console: `make run-local`

# Usage
Once the following instructions have been installed, you should be able to see the website in your folder. The database should be configured for you automatically.

For subsequent usages, one doesn't need to 

In your web browser, type in the URL:
`https://localhost:8000/static/login.html`

This should show you the login page for Claynce page.

# Contact
Please contact in Discord