# 🚀 A Near-Realworld Leptos Web App with Axum and SQLite Backend

<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>

This repository hosts a **Leptos demo application** developed as part of my exploration and experimentation with the **Rust/Leptos framework**. This project is more complex than the previously shared [ demo-tools-app-leptos-07-actix-tailwind](https://github.com/santhosh7403/demo-tools-app-leptos-07-actix-tailwind) and is intended to serve as a **more realistic, working example** for developers considering Leptos for their next project. I hope this hands-on code provides valuable insight into the framework's capabilities.

To facilitate a rapid setup, this version utilizes **SQLite** for the backend database. This choice is supported by the growing sentiment that SQLite is robust for many applications, particularly those not defined by heavy write operations [ recent opinions ](https://dev.to/shayy/everyone-is-wrong-about-sqlite-4gjf). To run the application, simply clone the project and follow the installation instructions below.

Before proceeding, you can view the application's functionality via the [ screenshots here ](https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite/blob/main/App_Screenshots.md).

A **PostgreSQL version** of this application, which features minor differences in the full-text search implementation, is also available [ here ](https://github.com/santhosh7403/realworld-app-leptos-axum).

---

## 🛠️ Key Technologies & Features

This application leverages the following core technologies and features:

* Leptos
* axum
* Server-Side Rendering (SSR)
* sqlite
* fts5 (Full-Text Search)
* Modal Windows
* argon2 (Password Encryption)
* uuid
* tailwindcss
* fontawesome icons


---

## ⚙️ Installation and Setup

**Prerequisites**

By default, `cargo-leptos` requires the **Rust nightly** toolchain and several cargo extensions. If you encounter issues, ensure these tools are installed. Consult the [ rustup documentation ](https://rustup.rs) for detailed instructions.

### Required Tools

Ensure the following Rust toolchains and dependencies are installed:


1.  `rustup toolchain install nightly --allow-downgrade` (Installs or ensures the **Rust nightly** toolchain is available)
2.  `rustup update` (Updates all installed Rust toolchains to their latest version)
3.  `rustup target add wasm32-unknown-unknown` (Adds the target necessary for compiling Rust to WebAssembly)
4.  `cargo install cargo-generate` (Installs the project templating tool)
5.  `cargo install cargo-leptos --locked` (Installs the essential Leptos build tool)


### Clone Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite.git]
cd realword-app-leptos-axum-sqlite
```


### Database Initialization

1. `source .env` - set the DATABASE_URL env variable

2, Follow the steps in [ README_DATABASE.md ](https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite/blob/main/README_DATABASE.md) to initialize the database schema and data.

### Run Application

You may now build and run the application:

```bash
cargo leptos watch 
# OR
cargo leptos serve
```

### Application access

Once the application has started successfully, access it via your web browser at [ localhost:3000 ](http://localhost:3000/)

Sample application screens.
<img width="1912" height="1031" alt="image" src="https://github.com/user-attachments/assets/582fa559-78a9-4f0c-804b-1e8b62415e28" />


<img width="1912" height="1031" alt="image" src="https://github.com/user-attachments/assets/63eeb201-34d4-4f97-b125-f4c2602fc220" />



<img width="1912" height="1031" alt="image" src="https://github.com/user-attachments/assets/85c8c011-f677-4a2b-9be5-4cd2c9360208" />



More screenshots are [ available here ](https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite/blob/main/App_Screenshots.md)

### Sample User Data

The application is pre-populated with sample users and data for immediate testing and demonstration.

1.   Available Users: user1 to user5

2.   Password: The password is the same as the username (e.g., user1 has a password of user1).

To remove this default data, delete the basedata files within the `./migrations` folder and follow the database setup steps outlined in the [README_DATABASE.md](https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite/blob/main/README_DATABASE.md).

### 🔍SQLite FTS5 (Full-Text Search) Implementation

The application features a robust full-text search capability powered by SQLite FTS5, which indexes three key fields from the `articles` table. For developers interested in the implementation or experimenting with different search methodologies, comprehensive documentation is available in the sqlite fts5 documentation[ here ](https://www.sqlite.org/fts5.html#overview_of_fts5)

## 🙏 Inspiration and Acknowledgements

The foundational structure of this application is derived from the realworld example by [Bechma/realworld-leptos](https://github.com/Bechma/realworld-leptos), with appreciation to any antecedent projects.

This particular version was initiated during the transition from Leptos 0.6 to 0.7 as a personal learning exercise. It has since undergone significant experimentation and refinement, including:

*   A complete user interface redesign utilizing tailwindcss and fontawesome icons.

*   Implementation of modal windows and re-wired page navigation.

*   Integration of SQLite FTS5 for comprehensive full-text search capabilities.

*   An updated, non-reloading pagination method for search results
