FROM golang:1.23.7

WORKDIR /usr/share/app

COPY go.mod go.sum ./

RUN go mod download

COPY . .

RUN go build -o build/fizzbuzz

EXPOSE 8080

ENTRYPOINT ["./build/fizzbuzz"]
CMD ["serve"]