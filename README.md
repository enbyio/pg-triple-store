# Triple Store using Rust & Postgres

## Design Considerations
Objects and Predicates are stored in a Table with Object and Predicate being assigned an ID. Subjects are also of Type Object in this case. \
Triples (Subject, Predicate, Object) are split into two different types. One containing the Relation between objects (both Subject and Object are ID's from the object table), the other containing the Properties (Object is a Value, stored as a string with the datatype stored too).

## Setup dev env
this is for using podman with postgres, requires podman to be installed
```sh
clone https://github.com/enbyio/pg-triple-store.git
cd pg-triple-store
chmod +x start_podman.sh
POSTGRES_PASSWORD=yourpassword ./start_podman.sh
echo "DATABASE_URL=postgres://postgres:yourpassword@localhost:5432/triple_store" > .env
```
