package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"sync"
)

func main() {
	var wg sync.WaitGroup

	// Server listens to communication from clients
	wg.Add(1)
	go func() {
		defer wg.Done()

		listener, err := net.Listen("tcp", "127.0.0.1:19389")
		if err != nil {
			fmt.Println("Error listening:", err)
			return
		}
		defer listener.Close()

		for {
			conn, err := listener.Accept()
			if err != nil {
				fmt.Println("Error accepting connection:", err)
				continue
			}

			go handleConnection(conn)
		}
	}()

	// Server sends work to clients
	instructions := [][2]string{
		{"3subredes.subred0.json", "192.168.3.1:19381"},
		{"3subredes.subred1.json", "192.168.3.2:19382"},
		{"3subredes.subred2.json", "192.168.3.3:19383"},
	}

	for _, instruction := range instructions {
		path := instruction[0]
		address := instruction[1]

		stream, err := net.Dial("tcp", address)
		if err != nil {
			fmt.Println("Error connecting to client:", err)
			continue
		}
		defer stream.Close()

		message := fmt.Sprintf("%s\n\n", path)
		_, err = stream.Write([]byte(message))
		if err != nil {
			fmt.Println("Error sending message to client:", err)
			continue
		}
	}

	wg.Wait()
}

func handleConnection(conn net.Conn) {
	defer conn.Close()

	bufReader := bufio.NewReader(conn)
	message := make([]string, 0)

	for {
		line, err := bufReader.ReadString('\n')
		if err != nil {
			fmt.Println("Error reading from client:", err)
			break
		}
		if line == "\n" {
			break
		}
		message = append(message, line)
	}

	fmt.Println(message)
}
