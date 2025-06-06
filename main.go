package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
)

// Config holds the configuration for the sub command
type Config struct {
	Name                string
	Root                string
	Color               string
	InferLongArguments  bool
}

// NewConfig creates a new configuration
func NewConfig(name, root, color string, inferLongArguments bool) *Config {
	return &Config{
		Name:               name,
		Root:               root,
		Color:              color,
		InferLongArguments: inferLongArguments,
	}
}

// LibexecPath returns the path to the libexec directory
func (c *Config) LibexecPath() string {
	return filepath.Join(c.Root, "libexec")
}

// UserCliCommand creates the command structure for user CLI
func (c *Config) UserCliCommand(name string) *cobra.Command {
	cmd := &cobra.Command{
		Use:   name,
		Short: "Dynamically generate rich CLIs from scripts",
		Long:  "sub is a tool for organizing scripts into a unified command-line interface",
	}

	cmd.Flags().Bool("usage", false, "Print usage")
	cmd.Flags().Bool("help", false, "Print help")
	cmd.Flags().Bool("completions", false, "Print completions")
	cmd.Flags().Bool("commands", false, "Print subcommands")
	cmd.Flags().Bool("validate", false, "Validate scripts")
	cmd.Flags().String("extension", "", "Filter subcommands by extension")

	return cmd
}

// UserCliMode represents the mode of operation
type UserCliMode int

const (
	UserCliModeInvoke UserCliMode = iota
	UserCliModeUsage
	UserCliModeHelp
	UserCliModeCommands
	UserCliModeCompletions
	UserCliModeValidate
)

// UserCliArgs holds parsed user CLI arguments
type UserCliArgs struct {
	Mode             UserCliMode
	CommandsWithArgs []string
	Extension        string
}

// Error types
type SubError struct {
	Type    string
	Message string
	Details []string
}

func (e SubError) Error() string {
	if len(e.Details) > 0 {
		return fmt.Sprintf("%s: %s\n  %s", e.Type, e.Message, strings.Join(e.Details, "\n  "))
	}
	return fmt.Sprintf("%s: %s", e.Type, e.Message)
}

func main() {
	config, cliArgs, err := parseSubCliArgs()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	userCliArgs, err := parseUserCliArgs(config, cliArgs)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	subcommand, err := findSubcommand(config, userCliArgs.CommandsWithArgs)
	if err != nil {
		handleError(config, err, userCliArgs.Mode == UserCliModeCompletions)
		return
	}

	switch userCliArgs.Mode {
	case UserCliModeInvoke:
		code, err := subcommand.Invoke()
		if err != nil {
			handleError(config, err, false)
			return
		}
		os.Exit(code)
	case UserCliModeUsage:
		usage, err := subcommand.Usage()
		if err != nil {
			handleError(config, err, false)
			return
		}
		fmt.Println(usage)
	case UserCliModeHelp:
		help, err := subcommand.Help()
		if err != nil {
			handleError(config, err, false)
			return
		}
		fmt.Println(help)
	case UserCliModeCommands:
		for _, sub := range subcommand.Subcommands() {
			if userCliArgs.Extension != "" {
				if ext := filepath.Ext(sub.Name()); ext != "" && ext[1:] == userCliArgs.Extension {
					fmt.Println(sub.Name())
				}
			} else {
				fmt.Println(sub.Name())
			}
		}
	case UserCliModeCompletions:
		code, err := subcommand.Completions()
		if err != nil {
			handleError(config, err, true)
			return
		}
		os.Exit(code)
	case UserCliModeValidate:
		errors := subcommand.Validate()
		for _, err := range errors {
			fmt.Printf("%s: %s\n", err.Path, err.Message)
		}
		if len(errors) == 0 {
			os.Exit(0)
		} else {
			os.Exit(1)
		}
	}
}

func parseSubCliArgs() (*Config, []string, error) {
	args := os.Args[1:]
	
	// Find the "--" separator
	dashDashIndex := -1
	for i, arg := range args {
		if arg == "--" {
			dashDashIndex = i
			break
		}
	}

	var subArgs, userArgs []string
	if dashDashIndex >= 0 {
		subArgs = args[:dashDashIndex]
		userArgs = args[dashDashIndex+1:]
	} else {
		subArgs = args
		userArgs = []string{}
	}

	// Parse sub arguments manually
	var name, absolute, executable, relative, color string
	var inferLongArguments bool
	color = "auto" // default value

	i := 0
	for i < len(subArgs) {
		arg := subArgs[i]
		
		switch {
		case arg == "--name":
			if i+1 >= len(subArgs) {
				return nil, nil, fmt.Errorf("--name requires a value")
			}
			name = subArgs[i+1]
			i += 2
		case arg == "--absolute":
			if i+1 >= len(subArgs) {
				return nil, nil, fmt.Errorf("--absolute requires a value")
			}
			absolute = subArgs[i+1]
			i += 2
		case arg == "--executable":
			if i+1 >= len(subArgs) {
				return nil, nil, fmt.Errorf("--executable requires a value")
			}
			executable = subArgs[i+1]
			i += 2
		case arg == "--relative":
			if i+1 >= len(subArgs) {
				return nil, nil, fmt.Errorf("--relative requires a value")
			}
			relative = subArgs[i+1]
			i += 2
		case arg == "--color":
			if i+1 >= len(subArgs) {
				return nil, nil, fmt.Errorf("--color requires a value")
			}
			color = subArgs[i+1]
			i += 2
		case arg == "--infer-long-arguments":
			inferLongArguments = true
			i++
		default:
			return nil, nil, fmt.Errorf("unknown argument: %s", arg)
		}
	}

	if name == "" {
		return nil, nil, fmt.Errorf("--name is required")
	}

	// Validate path arguments
	if absolute != "" && (executable != "" || relative != "") {
		return nil, nil, fmt.Errorf("cannot use --absolute with --executable or --relative")
	}

	if (executable != "" && relative == "") || (executable == "" && relative != "") {
		return nil, nil, fmt.Errorf("--executable and --relative must be used together")
	}

	if absolute == "" && executable == "" {
		return nil, nil, fmt.Errorf("must provide either --absolute or --executable with --relative")
	}

	var root string
	if absolute != "" {
		if !filepath.IsAbs(absolute) {
			return nil, nil, fmt.Errorf("--absolute path must be absolute")
		}
		root = absolute
	} else {
		execPath, err := filepath.Abs(executable)
		if err != nil {
			return nil, nil, fmt.Errorf("invalid executable path: %v", err)
		}
		root = filepath.Join(filepath.Dir(execPath), relative)
		root, err = filepath.Abs(root)
		if err != nil {
			return nil, nil, fmt.Errorf("invalid root path: %v", err)
		}
	}

	config := NewConfig(name, root, color, inferLongArguments)
	return config, userArgs, nil
}

func parseUserCliArgs(config *Config, cliArgs []string) (*UserCliArgs, error) {
	mode := UserCliModeInvoke
	extension := ""
	commandsWithArgs := cliArgs

	if len(cliArgs) > 0 {
		switch cliArgs[0] {
		case "--usage":
			mode = UserCliModeUsage
			commandsWithArgs = cliArgs[1:]
		case "--help":
			mode = UserCliModeHelp
			commandsWithArgs = cliArgs[1:]
		case "--commands":
			mode = UserCliModeCommands
			commandsWithArgs = cliArgs[1:]
			// Check for --extension flag
			if len(commandsWithArgs) > 0 && commandsWithArgs[0] == "--extension" {
				if len(commandsWithArgs) < 2 {
					return nil, fmt.Errorf("--extension requires a value")
				}
				extension = commandsWithArgs[1]
				commandsWithArgs = commandsWithArgs[2:]
			}
		case "--completions":
			mode = UserCliModeCompletions
			commandsWithArgs = cliArgs[1:]
		case "--validate":
			mode = UserCliModeValidate
			commandsWithArgs = cliArgs[1:]
		}
	}

	return &UserCliArgs{
		Mode:             mode,
		CommandsWithArgs: commandsWithArgs,
		Extension:        extension,
	}, nil
}

// Command interface
type Command interface {
	Name() string
	Summary() string
	Usage() (string, error)
	Help() (string, error)
	Subcommands() []Command
	Completions() (int, error)
	Invoke() (int, error)
	Validate() []ValidationError
}

// ValidationError represents a validation error
type ValidationError struct {
	Path    string
	Message string
}

func findSubcommand(config *Config, commandsWithArgs []string) (Command, error) {
	libexecPath := config.LibexecPath()
	
	// Check if libexec directory exists
	if info, err := os.Stat(libexecPath); err != nil || !info.IsDir() {
		return nil, SubError{Type: config.Name, Message: "libexec directory not found in root"}
	}

	if len(commandsWithArgs) == 0 {
		return &DirectoryCommand{
			names:  []string{},
			path:   libexecPath,
			config: config,
		}, nil
	}

	path := libexecPath
	names := []string{}
	remaining := commandsWithArgs[:]

	for len(remaining) > 0 {
		head := remaining[0]
		
		// Don't allow commands starting with '.'
		if strings.HasPrefix(head, ".") {
			return nil, SubError{Type: config.Name, Message: fmt.Sprintf("no such sub command '%s'", head)}
		}

		nextPath := filepath.Join(path, head)
		
		if _, err := os.Stat(nextPath); os.IsNotExist(err) {
			return nil, SubError{Type: config.Name, Message: fmt.Sprintf("no such sub command '%s'", head)}
		}

		names = append(names, head)
		remaining = remaining[1:]

		if info, err := os.Stat(nextPath); err == nil {
			if info.IsDir() {
				path = nextPath
				if len(remaining) == 0 {
					// Directory command
					return &DirectoryCommand{
						names:  names,
						path:   path,
						config: config,
					}, nil
				}
				continue
			} else {
				// File command
				return &FileCommand{
					names:  names,
					path:   nextPath,
					args:   remaining,
					config: config,
				}, nil
			}
		}
	}

	// Should not reach here
	return &DirectoryCommand{
		names:  names,
		path:   path,
		config: config,
	}, nil
}

// DirectoryCommand implements Command for directories
type DirectoryCommand struct {
	names  []string
	path   string
	config *Config
}

func (d *DirectoryCommand) Name() string {
	if len(d.names) == 0 {
		return d.config.Name
	}
	return d.names[len(d.names)-1]
}

func (d *DirectoryCommand) Summary() string {
	return "Directory command"
}

func (d *DirectoryCommand) Usage() (string, error) {
	if len(d.names) == 0 {
		return fmt.Sprintf("Usage: %s [args]...", d.config.Name), nil
	}
	return fmt.Sprintf("Usage: %s %s [args]...", d.config.Name, strings.Join(d.names, " ")), nil
}

func (d *DirectoryCommand) Help() (string, error) {
	usage, err := d.Usage()
	if err != nil {
		return "", err
	}
	
	help := usage + "\n\nAvailable subcommands:\n"
	for _, sub := range d.Subcommands() {
		help += fmt.Sprintf("    %s\n", sub.Name())
	}
	
	return help, nil
}

func (d *DirectoryCommand) Subcommands() []Command {
	var commands []Command
	
	entries, err := os.ReadDir(d.path)
	if err != nil {
		return commands
	}
	
	for _, entry := range entries {
		name := entry.Name()
		
		// Skip hidden files
		if strings.HasPrefix(name, ".") {
			continue
		}
		
		fullPath := filepath.Join(d.path, name)
		names := append(d.names, name)
		
		if entry.IsDir() {
			commands = append(commands, &DirectoryCommand{
				names:  names,
				path:   fullPath,
				config: d.config,
			})
		} else {
			// Check if file is executable
			if info, err := os.Stat(fullPath); err == nil {
				if info.Mode()&0111 != 0 {
					commands = append(commands, &FileCommand{
						names:  names,
						path:   fullPath,
						args:   []string{},
						config: d.config,
					})
				}
			}
		}
	}
	
	return commands
}

func (d *DirectoryCommand) Completions() (int, error) {
	for _, sub := range d.Subcommands() {
		fmt.Println(sub.Name())
	}
	return 0, nil
}

func (d *DirectoryCommand) Invoke() (int, error) {
	help, err := d.Help()
	if err != nil {
		return 1, err
	}
	fmt.Println(help)
	return 0, nil
}

func (d *DirectoryCommand) Validate() []ValidationError {
	return []ValidationError{}
}

// FileCommand implements Command for executable files
type FileCommand struct {
	names  []string
	path   string
	args   []string
	config *Config
}

func (f *FileCommand) Name() string {
	return f.names[len(f.names)-1]
}

func (f *FileCommand) Summary() string {
	// TODO: Extract summary from file comments
	return ""
}

func (f *FileCommand) Usage() (string, error) {
	// TODO: Extract usage from file comments
	if len(f.names) == 0 {
		return fmt.Sprintf("Usage: %s [args]...", f.config.Name), nil
	}
	return fmt.Sprintf("Usage: %s %s [args]...", f.config.Name, strings.Join(f.names, " ")), nil
}

func (f *FileCommand) Help() (string, error) {
	usage, err := f.Usage()
	if err != nil {
		return "", err
	}
	return usage, nil
}

func (f *FileCommand) Subcommands() []Command {
	return []Command{}
}

func (f *FileCommand) Completions() (int, error) {
	// TODO: Implement completions
	return 0, nil
}

func (f *FileCommand) Invoke() (int, error) {
	// Set environment variables
	envName := fmt.Sprintf("_%s_ROOT", strings.ToUpper(f.config.Name))
	os.Setenv(envName, f.config.Root)
	
	// TODO: Parse arguments and set _MAIN_ARGS
	argsEnvName := fmt.Sprintf("_%s_ARGS", strings.ToUpper(f.config.Name))
	os.Setenv(argsEnvName, strings.Join(f.args, " "))
	
	// Execute the script
	cmd := exec.Command(f.path, f.args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	
	if err := cmd.Run(); err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			return exitError.ExitCode(), nil
		}
		return 1, err
	}
	
	return 0, nil
}

func (f *FileCommand) Validate() []ValidationError {
	return []ValidationError{}
}

func handleError(config *Config, err error, silent bool) {
	if !silent {
		if subErr, ok := err.(SubError); ok {
			if subErr.Type == config.Name {
				fmt.Fprintf(os.Stderr, "%s\n", subErr.Message)
			} else {
				fmt.Fprintf(os.Stderr, "%s: %s\n", subErr.Type, subErr.Message)
			}
		} else {
			fmt.Fprintf(os.Stderr, "%s: %v\n", config.Name, err)
		}
	}
	os.Exit(1)
}