# sql-rs

An in-memory SQL database built for the purposes of learning the
[pest](https://github.com/pest-parser/pest) parser generator.

Some examples to try:

```
$ cargo run --bin sql-rs -- -s test-files/seed.sql
...
> select * from users;
> select u.email, a.* from users as u join addresses as a on u.id = a.user_id where u.age > 22 and a.type = 'mail';
> select * from users as u left join addresses as a on false;
> select * from users as u right join addresses as a on false;
> select * from users order by age desc;
> select * from addresses order by street1 limit 2;
> select count(*) from addresses;
> update users set age = 22 where id = '1';
```

Implemented functionality is a basic subset of SQL. What's implemented mostly
works as expected and isn't performant.

Implemented:
  * datatypes number, text, and boolean.
  * create table without primary keys
  * insert a single row. all columns required.
  * select queries
    * inner, left, and right joins
    * WHERE filters
    * ORDER BY (ASC/DESC)
    * LIMIT
    * count(*)
  * single table, all literal update

Want to implement:
  * GROUP BY
  * subqueries
  * query planning / non-full table scans
  * primary keys
  * indexes
  * proper aggregate functions (sum, max) count(something other than *)

I'd also like to write a small markdown book (more of a journal) documenting my
approach and learnings.
