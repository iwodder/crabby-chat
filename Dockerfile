#Stage 1 - Copy and compile UI
FROM node:15.7.0-alpine3.10 as UI
WORKDIR /usr/src/app
COPY /client/package*.json ./
RUN npm install
COPY /client .
RUN npm run build

#Stage 2 - Pull in UI and setup Rust web-server
FROM rust:1.48
RUN rustup default nightly
WORKDIR /usr/src/CrabbyChat
COPY /server .
COPY --from=UI /usr/src/app/dist/client ./static/
RUN cargo install --path .
EXPOSE 8000
CMD ["server"]
