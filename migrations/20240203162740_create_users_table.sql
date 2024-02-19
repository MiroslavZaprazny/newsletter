CREATE TABLE users(
    id uuid PRIMARY KEY,
    username varchar(255) NOT NULL UNIQUE,
    password TEXT NOT NULL
);
