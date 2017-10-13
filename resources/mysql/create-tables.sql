CREATE TABLE Users (
   id         INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   slack_id   VARCHAR(9) NOT NULL UNIQUE,
   nickname   VARCHAR(64),
   first_name VARCHAR(64),
   last_name  VARCHAR(64),
   email      VARCHAR(64),
   phone      VARCHAR(64)
);

CREATE TABLE Karma (
   recipient INT UNSIGNED REFERENCES Users(id),
   donor     INT UNSIGNED REFERENCES Users(id),
   points    INT
);

CREATE INDEX UserIndex ON Users(slack_id);
