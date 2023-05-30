FROM node:16
WORKDIR /usr/src/app
COPY ./server/. .
COPY ./proto/publish.proto .
RUN npm install
EXPOSE 8080 50051
ENV NODE_ENV production
CMD [ "npm", "start" ]
