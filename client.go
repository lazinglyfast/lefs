package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strconv"
	"time"
)

func main() {
	args := os.Args
	address := args[1]
	serverAddress := args[2]
	lastCycle, _ := strconv.Atoi(args[3])
	listener, _ := net.Listen("tcp", address)

	for {
		conn, _ := listener.Accept()
		go handleConnection(conn, lastCycle, address, serverAddress)
	}
}

func handleConnection(conn net.Conn, lastCycle int, address string, serverAddress string) {
	defer conn.Close()
	reader := bufio.NewReader(conn)
	message := make([]string, 0)

	for {
		line, _ := reader.ReadString('\n')
		if line == "\n" {
			break
		}
		message = append(message, line)
	}

	if len(message) > 0 {
		path := message[0]
		fmt.Printf("%v", path)
		lefs, _ := NewLefs(path)
		engine := NewEngine(*lefs)
		engine.Simulate(0, lastCycle)

		serverConn, _ := net.Dial("tcp", serverAddress)
		serverMessage := fmt.Sprintf("%s processed petri network %s\n\n", address, path)
		serverConn.Write([]byte(serverMessage))
		serverConn.Close()
	}
}

// Engine struct definition
type Engine struct {
	Cycle                 int
	Lefs                  Lefs
	Events                []Event
	Logs                  []Log
	EventCount            int
	EstimulatedTransitions []int
}

// NewEngine creates a new Engine instance
func NewEngine(lefs Lefs) *Engine {
	return &Engine{
		Cycle:                 0,
		Lefs:                  lefs,
		Events:                make([]Event, 0),
		Logs:                  make([]Log, 0),
		EventCount:            0,
		EstimulatedTransitions: make([]int, 0),
	}
}

// Simulate simulates the engine for a given range of cycles
func (e *Engine) Simulate(firstCycle, lastCycle int) {
	start := time.Now()
	e.Cycle = firstCycle

	for e.Cycle < lastCycle {
		fmt.Printf("RELOJ LOCAL !!!  = %d\n", e.Cycle)
		fmt.Println(e.Lefs)
		e.Step(lastCycle)
	}

	fmt.Printf("event_count: %d\n", e.EventCount)
	elapsed := time.Since(start)
	fmt.Printf("elapsed: %v microseconds\n", elapsed.Nanoseconds()/1000)
}

// Step performs a simulation step
func (e *Engine) Step(lastCycle int) {
	e.Pep()

	fmt.Println("-----------Stack de transiciones sensibilizadas---------")
	fmt.Println(e.EstimulatedTransitions)
	fmt.Println("-----------Final Stack de transiciones---------")

	for len(e.EstimulatedTransitions) > 0 {
		estimulatedTransitionIndex := e.EstimulatedTransitions[len(e.EstimulatedTransitions)-1]
		e.EstimulatedTransitions = e.EstimulatedTransitions[:len(e.EstimulatedTransitions)-1]

		e.Fire(estimulatedTransitionIndex)
		e.Logs = append(e.Logs, Log{EstimulatedTransitionIndex: estimulatedTransitionIndex, Cycle: e.Cycle})
	}

	fmt.Println("-----------Lista eventos después de disparos---------")
	fmt.Println("Estructura EventList")
	for i, event := range e.Events {
		fmt.Printf("  Evento -> %d\n", i)
		fmt.Println(event)
	}
	fmt.Println("-----------Final lista eventos---------")

	if len(e.Events) > 0 {
		e.Cycle = e.Events[0].Cycle
	} else {
		e.Cycle = lastCycle
	}
	fmt.Printf("NEXT CLOCK...... : %d\n", e.Cycle)

	e.Aftermath()
}

// Pep updates the list of estimulated transitions
func (e *Engine) Pep() {
	for i, transition := range e.Lefs.Transitions {
		if transition.Constant <= 0 && transition.Cycle == e.Cycle {
			e.EstimulatedTransitions = append(e.EstimulatedTransitions, i)
		}
	}
}

// Fire performs the firing of a transition
func (e *Engine) Fire(estimulatedTransitionIndex int) {
	transition := e.Lefs.Transitions[estimulatedTransitionIndex]

	for _, payload := range transition.IulPayloads {
		e.Lefs.Transitions[payload.TransitionIndex].Constant += payload.Constant
	}

	cycle := transition.Cycle + transition.Duration
	for _, payload := range transition.PulPayloads {
		event := Event{
			Cycle:            cycle,
			TransitionIndex: payload.TransitionIndex,
			Constant:         payload.Constant,
		}
		e.Events = append([]Event{event}, e.Events...)
	}
}

// Aftermath processes events after firing transitions
func (e *Engine) Aftermath() {
	for len(e.Events) > 0 {
		event := e.Events[0]
		e.Events = e.Events[1:]

		e.Lefs.Transitions[event.TransitionIndex].Constant += event.Constant
		e.Lefs.Transitions[event.TransitionIndex].Cycle = event.Cycle
		e.EventCount++
	}
}

// Event struct definition
type Event struct {
	Cycle           int
	TransitionIndex int
	Constant        int
}

// Log struct definition
type Log struct {
	EstimulatedTransitionIndex int
	Cycle                     int
}

// Lefs struct definition
type Lefs struct {
	Transitions                 []Transition
	EstimulatedTransitionIndices []int
}

// NewLefs creates a new Lefs instance
func NewLefs(path string) (*Lefs, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	lefs, err := parseLefs(file)
	if err != nil {
		return nil, err
	}

	return lefs, nil
}

// parseLefs parses the contents of the LEFS file
func parseLefs(file *os.File) (*Lefs, error) {
	var lefsJson LefsJson
	if err := json.NewDecoder(file).Decode(&lefsJson); err != nil {
		return nil, err
	}

	transitions := make([]Transition, len(lefsJson.IARed))
	for i, transitionJson := range lefsJson.IARed {
		iulPayloads := make([]Payload, len(transitionJson.IIListactesIUL))
		for j, payloadJson := range transitionJson.IIListactesIUL {
			iulPayloads[j] = Payload{TransitionIndex: payloadJson.TransitionIndex, Constant: payloadJson.Constant}
		}

		pulPayloads := make([]Payload, len(transitionJson.IIListactesPUL))
		for j, payloadJson := range transitionJson.IIListactesPUL {
			pulPayloads[j] = Payload{TransitionIndex: payloadJson.TransitionIndex, Constant: payloadJson.Constant}
		}

		transitions[i] = Transition{
			ID:            transitionJson.IIIDGlobal,
			Constant:      transitionJson.IIValor,
			Cycle:         transitionJson.IITiempo,
			Duration:      transitionJson.IIDuracionDisparo,
			IulPayloads:   iulPayloads,
			PulPayloads:   pulPayloads,
		}
	}

	lefs := &Lefs{
		Transitions:                 transitions,
		EstimulatedTransitionIndices: make([]int, 0),
	}

	return lefs, nil
}

// LefsJson struct definition for JSON parsing
type LefsJson struct {
	IARed []TransitionJson `json:"ia_red"`
}

// TransitionJson struct definition for JSON parsing
type TransitionJson struct {
	IIIDGlobal           int             `json:"ii_idglobal"`
	IIValor              int             `json:"ii_valor"`
	IITiempo             int             `json:"ii_tiempo"`
	IIDuracionDisparo    int             `json:"ii_duracion_disparo"`
	IIListactesIUL       []PayloadJson   `json:"ii_listactes_IUL"`
	IIListactesPUL       []PayloadJson   `json:"ii_listactes_PUL"`
}

// PayloadJson struct definition for JSON parsing
type PayloadJson struct {
	TransitionIndex int `json:"transition_index"`
	Constant        int `json:"constant"`
}

// Transition struct definition
type Transition struct {
	ID           int
	Constant     int
	Cycle        int
	Duration     int
	IulPayloads  []Payload
	PulPayloads  []Payload
}

// Payload struct definition
type Payload struct {
	TransitionIndex int
	Constant        int
}

func (l *Lefs) String() string {
	result := "STRUCT LEFS\n"
	result += fmt.Sprintf("\tNº transiciones: %d\n", len(l.Transitions))
	result += "------Lista transiciones---------\n"
	for _, transition := range l.Transitions {
		result += fmt.Sprintf("%s", transition)
	}
	result += "------Final lista transiciones---------\n"
	result += "FINAL ESTRUCTURA LEFS"
	return result
}

func (t *Transition) String() string {
	result := "Dato Transicion:\n"
	result += fmt.Sprintf("IDLOCALTRANSICION: %d\n", t.ID)
	result += fmt.Sprintf(" VALOR LEF: %d\n", t.Constant)
	result += fmt.Sprintf(" TIEMPO: %d\n", t.Cycle)
	result += fmt.Sprintf(" DURACION DISPARO: %d\n", t.Duration)
	result += " LISTA DE CTES IUL: \n"
	for _, payload := range t.IulPayloads {
		result += fmt.Sprintf("\tTRANSICION: %d\t\tCTE: %d\n", payload.TransitionIndex, payload.Constant)
	}
	result += " LISTA DE CTES PUL: \n"
	for _, payload := range t.PulPayloads {
		result += fmt.Sprintf("\tTRANSICION: %d\t\tCTE: %d\n", payload.TransitionIndex, payload.Constant)
	}
	return result
}

func (e *Event) String() string {
	result := fmt.Sprintf("    TIEMPO: %d\n", e.Cycle)
	result += fmt.Sprintf("    TRANSICION: %d\n", e.TransitionIndex)
	result += fmt.Sprintf("    CONSTANTE: %d\n", e.Constant)
	return result
}

