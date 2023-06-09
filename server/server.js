//listens for external requests on port 50010
const express = require("express");
const grpc = require("@grpc/grpc-js");
const protoLoader = require("@grpc/proto-loader");
const bodyParser = require("body-parser");
const path = require("path");

const app = express();
const protoPath = path.join(__dirname, "publish.proto");
const packageDefinition = protoLoader.loadSync(protoPath);
const publishProto = grpc.loadPackageDefinition(packageDefinition).publish;
//assuming producer is running on port 50090
const client = new publishProto.MyAPIService(
  "producer-service:50090",
  grpc.credentials.createInsecure(),
);

// Parse JSON request bodies
app.use(bodyParser.json());

// API endpoint to receive data from users
app.get("/data", (req, res) => {
  // const requestData = req.body.data; // Assuming the data is sent in the request body

  // const request = new publishProto.express_data_to_producer();
  // request.setData(requestData);
  console.log(JSON.stringify(req.query.data));

  // Call the gRPC client to send data to the producer
  client.expressToProducer(req.query.data, (err, response) => {
    if (err) {
      console.error("Error:", err);
      res.status(500).send("Internal Server Error");
    } else if (response?.result?.response_to_express == 0) {
      console.log("Result: Unsuccessful processing");
      res.send("Data not processed");
    } else {
      console.log("Result:", response?.result?.response_to_express);
      res.send("Data processed successfully");
    }
  });
});

// Start the Express server
app.listen(50010, () => {
  console.log("API server is running on port 50010");
});
