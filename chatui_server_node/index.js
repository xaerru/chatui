const net = require("net");

const LOCAL = 3000;

const clients = [];

const server = net.createServer((client) => {
  clients.push(client);

  console.log(`Client 127.0.0.1:${client.remotePort} connected.`);

  client.on("data", (data) => {
    console.log(data.toString());

    clients.forEach((client) => client.write(data.toString()));
  });

  client.on("end", () => {
    console.log(`Closing connection with 127.0.0.1:${client.remotePort}`);
  });
});

server.listen(LOCAL, () => {
  console.log(`Listening on 127.0.0.1:${LOCAL}`);
});
