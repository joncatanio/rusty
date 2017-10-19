CREATE TABLE Users (
   id         INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   slack_id   VARCHAR(16) NOT NULL UNIQUE,
   deleted    BOOLEAN DEFAULT FALSE,
   created    DATETIME
);

CREATE TABLE Karma (
   id        INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   recipient INT UNSIGNED REFERENCES Users(id),
   donor     INT UNSIGNED REFERENCES Users(id),
   points    INT NOT NULL,
   created   DATETIME
);

CREATE INDEX UserIndex ON Users(slack_id);
