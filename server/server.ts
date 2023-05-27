const express = require('express');
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const bodyParser = require('body-parser');
const path = require('path');

const app = express();
const protoPath = path.join(__dirname, '../proto/publish.proto');
const packageDefinition = protoLoader.loadSync(protoPath);
const publishProto = grpc.loadPackageDefinition(packageDefinition).publish;

const client = new publishProto.MyAPIService('localhost:50051', grpc.credentials.createInsecure());

// Parse JSON request bodies
app.use(bodyParser.json());

// API endpoint to receive data from users
app.post('/data', (req, res) => {
    const requestData = req.body.data; // Assuming the data is sent in the request body
  
    const request = new publishProto.ExpressDataToProducer();
    request.setData(requestData);
  
    // Call the gRPC client to send data to the producer
    client.expressToProducer(request, (err, response) => {
      if (err) {
        console.error('Error:', err);
        res.status(500).send('Internal Server Error');
      } else {
        console.log('Result:', response.response_to_express);
        res.send('Data processed successfully');
      }
    });
  });
  
  // Start the Express server
  app.listen(3000, () => {
    console.log('API server is running on port 3000');
  });