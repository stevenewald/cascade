import express from 'express';
import * as grpc from '@grpc/grpc-js';
import bodyParser from 'body-parser';
import protoLoader from '@grpc/proto-loader';
import { MyAPIService } from '../proto/publish.proto';
import { ExpressDataToProducer, ProducerToExpressAck } from '../proto/publish.proto';

const protoPath = '../proto/publish.proto'; // Update with the correct path
const packageDefinition = protoLoader.loadSync(protoPath);
const publishProto = grpc.loadPackageDefinition(packageDefinition).publish;

const app = express();
const client = new MyAPIService('localhost:50051', grpc.credentials.createInsecure());

// API endpoint to receive data from users
app.post('/data', (req, res) => {
    const requestData = req.body.data; // Assuming the data is sent in the request body
  
    const request = new ExpressDataToProducer();
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