# Rust Rewrite Plan
## 1. Removing old code
I want to remove every folder that contains static assets and templates from the old frontend.
❌ /static
❌ /templates
Removed

## 2. Move old code for preservation
The /libs folder contains valuable JS code, that belongs to the backend.
It is moved to /backend/libs_js

## 2.1 Remove old config files
eslint configs and similar JS related config files can be removed

## 3. Sketch out backend folder structure, technologies, document backend
### Technologies
1. actix_web
2. actix_session with feature cookie-store

## Folder Structure
- src/
    - main.rs: Calls server and connects pages
    - module_name/
        - module.rs
## Backend Documentation

