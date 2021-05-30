create table users (
  id    text,
  email text,
  age   number
);

insert into users (id, email, age) values ('1', 'user1@email.com', 21);
insert into users (id, email, age) values ('2', 'user2@email.com', 22);
insert into users (id, email, age) values ('3', 'user3@email.com', 23);

create table addresses (
  id      text,
  user_id text,
  type    text
);

insert into addresses (id, user_id, type) values ('1', '1', 'home');
insert into addresses (id, user_id, type) values ('2', '1', 'mail');
