# Development stage
FROM node:22.17.0-alpine AS development
# Install build dependencies for native modules and pnpm
RUN apk add --no-cache python3 py3-pip make g++ gcc
RUN corepack enable && corepack prepare pnpm@10.12.1 --activate
WORKDIR /app

# Copy package files and install dependencies
COPY package.json pnpm-lock.yaml* pnpm-workspace.yaml* ./
RUN pnpm install --frozen-lockfile

# Copy source code
COPY . .

EXPOSE 4001
CMD ["pnpm", "run", "dev"]

# Build stage
FROM node:22.17.0-alpine AS build
# Install build dependencies for native modules and pnpm
RUN apk add --no-cache python3 py3-pip make g++ gcc
RUN corepack enable && corepack prepare pnpm@10.12.1 --activate
WORKDIR /app

# Copy package files and install dependencies
COPY package.json pnpm-lock.yaml* pnpm-workspace.yaml* ./
RUN pnpm install --frozen-lockfile

# Copy source code and build
COPY . .
RUN pnpm run build

# Production stage
FROM nginx:1.28.0-alpine AS production
COPY --from=build /app/dist /usr/share/nginx/html

# Create basic nginx config
RUN echo 'server {\n\
    listen 80;\n\
    server_name localhost;\n\
    root /usr/share/nginx/html;\n\
    index index.html;\n\n\
    location / {\n\
        try_files $uri $uri/ /index.html;\n\
    }\n\
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]