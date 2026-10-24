create schema audit;

create table audit.events (
    id int8 primary key,
    user_id int8 references users (id),
    kind text
);

create domain positive_int as int4 check (value > 0);

create table posts (
    id uuid primary key,
    author_id int8 not null references users (id),
    editor_id int8 references users (id),
    score positive_int not null,
    roles member_role[] not null default '{}',
    price numeric(10, 2)
);

create view active_users as select id, email, team_id from users where display_name is not null;
