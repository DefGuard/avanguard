FROM node:19.4.0-alpine3.16 AS builder

RUN apk add git

WORKDIR /build

COPY ./package.json ./pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install

COPY ./ .
RUN pnpm build

FROM nginx:1.23.3-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /build/dist /usr/share/nginx/html
