FROM node:current-alpine as build

WORKDIR /app

ENV PATH /app/node_modules/.bin:$PATH

COPY package.json ./
COPY yarn.lock ./
RUN yarn

COPY . ./

RUN yarn build

# production environment
# FROM nginx:stable-alpine
# COPY ./nginx.conf /etc/nginx/nginx.conf
# COPY --from=build /app/build /usr/share/nginx/html
# EXPOSE 80
# CMD ["nginx", "-g", "daemon off;"]

FROM node:current-alpine

WORKDIR /app

COPY --from=build /app/build ./
CMD npx serve -s . -p 80