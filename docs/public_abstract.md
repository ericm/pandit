**Pandit** transfers data between applications on different servers.
It does this by running an instance of itself on each server and listening for data requests by applications on that server.

There are two types of applications that run on the servers: clients and 'servers'.
Clients query data and 'servers' provide said data.
Pandit acts as a middle-man for these queries.

## Services

Pandit provides 'services' to clients. A service is an abstraction on top of a 'server' that
just focuses on the data provided by the server.

Services are defined in a file that both the client and Pandit have access to, so both know how to parse data for a particular service.

Pandit handles the querying and conversion of data from the servers, allowing the clients to receive data in a uniform format.

## Caching

Since Pandit knows how to translate the data, it also knows when it gets the same request for data twice.
This allows it to store previously seen responses. If it gets the same request twice (based on a stategy defined by the user in the service file), it will return the data there and then, without adding extra load to the server.
