-- The schema the typed-query tests describe their statements against (run by run.sh).
drop schema public cascade;
create schema public;

create type mood as enum ('sad', 'ok', 'happy');

create table users (
    id bigserial primary key,
    email text not null,
    display_name text,
    created_at timestamptz not null default now(),
    settings jsonb,
    mood mood not null default 'ok',
    score int4 not null default 0,
    tags text[] not null default '{}'
);

create table posts (
    id bigserial primary key,
    author_id bigint not null references users (id),
    title text not null,
    body text,
    created_at timestamptz not null default now()
);

create view user_posts as
    select u.id as user_id, u.email, p.id as post_id
    from users u join posts p on p.author_id = u.id;

insert into users (email, display_name, settings, tags, mood)
values ('a@example.com', 'Alice', '{"theme": "dark", "font_size": 16}', '{x,y}', 'happy'),
       ('b@example.com', null, null, default, default);

insert into posts (author_id, title, body) values (1, 'hello', 'first'), (1, 'second', null);
