

<div style="display: flex; align-items: center; justify-content: center;">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="../img/beetl_logo_dark.svg">
        <source media="(prefers-color-scheme: light)" srcset="../img/beetl_logo.svg">
        <img src="../img/beetl_logo.svg" width="200" alt="beetl logo">
    </picture>
    <span style="font-size: 48px; margin: 0 20px; font-weight: regular; font-family: Open Sans, sans-serif;"> + </span>
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://companieslogo.com/img/obeetl/MDB_BIG.D-96d632a9.png?t=1720244492">
        <source media="(prefers-color-scheme: light)" srcset="https://cdn.iconscout.com/icon/free/png-256/free-mongodb-logo-icon-download-in-svg-png-gif-file-formats--wordmark-programming-langugae-freebies-pack-logos-icons-1175140.png?f=webp&w=256">
        <img src="https://cdn.iconscout.com/icon/free/png-256/free-mongodb-logo-icon-download-in-svg-png-gif-file-formats--wordmark-programming-langugae-freebies-pack-logos-icons-1175140.png?f=webp&w=256" width="200" alt="MongoDB logo">
    </picture>
</div>

<br><br>

## beetl-MongoDB
This companion crate implements a beetl vector store based on MongoDB.

## Usage

Add the companion crate to your `Cargo.toml`, along with the beetl-core crate:

```toml
[dependencies]
beetl-mongodb = "0.1.3"
beetl-core = "0.4.0"
```

You can also run `cargo add beetl-mongodb beetl-core` to add the most recent versions of the dependencies to your project.

See the [`/examples`](./examples) folder for usage examples.
