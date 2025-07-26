FROM node:22.17.0-alpine AS base
# Install build dependencies for native modules and pnpm
RUN apk add --no-cache python3 py3-pip make g++ gcc
RUN corepack enable && corepack prepare pnpm@10.12.1 --activate
WORKDIR /app
COPY package.json pnpm-lock.yaml* ./
RUN pnpm install --prod --frozen-lockfile

# Development stage
FROM node:22.17.0-alpine AS development
# Install build dependencies for native modules and pnpm
RUN apk add --no-cache python3 py3-pip make g++ gcc
RUN corepack enable && corepack prepare pnpm@10.12.1 --activate
WORKDIR /app
COPY package.json pnpm-lock.yaml* .npmrc* ./
RUN pnpm install --frozen-lockfile
COPY . .
EXPOSE 4001
CMD ["pnpm", "run", "dev"]

# Build stage
FROM base AS build
WORKDIR /app
COPY package.json pnpm-lock.yaml* .npmrc* ./
RUN pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

# Production stage
FROM nginx:1.28.0-alpine AS production
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]