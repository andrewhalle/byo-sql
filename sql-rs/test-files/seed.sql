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
  type    text,
  street1 text
);

insert into addresses (id, user_id, type, street1) values ('1', '1', 'home', '123 Test Ave.');
insert into addresses (id, user_id, type, street1) values ('2', '1', 'mail', '321 Test Ave.');

insert into addresses (id, user_id, type, street1) values ('3', '2', 'home', '3838 Testing St.');
insert into addresses (id, user_id, type, street1) values ('4', '2', 'mail', '3838 Testing St.');

insert into addresses (id, user_id, type, street1) values ('5', '3', 'home', '29389 Mock Way');
insert into addresses (id, user_id, type, street1) values ('6', '3', 'mail', '3839 Testing St.');
