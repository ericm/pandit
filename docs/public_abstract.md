**Pandit** provides a way for data to be transferred between applications on different servers.
It does this by running an instance of itself on each server.

There are two types of applications that run on the servers: clients and 'servers'.
Clients query data and 'servers' provide said data.
Pandit acts as a middle-man for these queries.

## Services

Clients can be built to query a Service.
Each service is defined in a file called a _Proto_ file.
It provides an interface in which a client can call for a piece of data on a server.
For example, the _Proto_ file can provide a `GetUser` function that will allow clients
to query information on a user.

## Servers and Caching

Pandit will translate these requests into the whatever format the server requires.

Since Pandit knows how to translate the data, it also knows when it gets the same request for data twice.
This allows it to introduce the concept of caching to the queries. If it gets the same request twice (based on a stategy defined by the user in the _Proto_ file), it will return the data there and then, without adding extra load to the server.
