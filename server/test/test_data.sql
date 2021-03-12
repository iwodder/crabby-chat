BEGIN;
CREATE TABLE users(
    id INTEGER PRIMARY KEY,
    user_id TEXT UNIQUE,
    user_name TEXT);

CREATE TABLE favorites(
    id INTEGER PRIMARY KEY,
    user_id TEXT,
    name TEXT,
    FOREIGN KEY(user_id) REFERENCES users (user_id));

INSERT INTO users (user_id, user_name) VALUES ("abcd-1234", "jhalpert"), ("bcde-2345", "mscott");
INSERT INTO favorites (user_id, name) VALUES ("abcd-1234", "dunmifsys"),
                                             ("abcd-1234", "bigtuna"),
                                             ("bcde-2345", "scotts-tots");
COMMIT;