create table mig_teams (
    id int8 generated always as identity primary key,
    name text not null -- a comment; with a semicolon
);
insert into mig_teams (name) values ('it''s; fine');
