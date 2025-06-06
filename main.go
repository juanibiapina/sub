package main

import (
	"bufio"
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
		fmt.Fprintf(os.Stderr, "%v\n", err)
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
	commandsWithArgs := []string{}
	
	i := 0
	for i < len(cliArgs) {
		arg := cliArgs[i]
		
		if arg == "--usage" {
			mode = UserCliModeUsage
		} else if arg == "--help" || arg == "-h" {
			mode = UserCliModeHelp
		} else if arg == "--commands" {
			mode = UserCliModeCommands
		} else if arg == "--completions" {
			mode = UserCliModeCompletions
		} else if arg == "--validate" {
			mode = UserCliModeValidate
		} else if arg == "--extension" {
			if i+1 >= len(cliArgs) {
				return nil, fmt.Errorf("--extension requires a value")
			}
			extension = cliArgs[i+1]
			i++ // skip the value
		} else if strings.HasPrefix(arg, "--extension=") {
			extension = arg[12:] // remove "--extension="
		} else {
			// This is a command or argument
			commandsWithArgs = append(commandsWithArgs, arg)
		}
		i++
	}
	
	// Handle conflicts
	if mode == UserCliModeHelp && len(cliArgs) > 0 && (cliArgs[0] == "--usage" || strings.Contains(strings.Join(cliArgs, " "), "--usage")) {
		return nil, fmt.Errorf("error: the argument '--usage' cannot be used with '--help'\n\nUsage: %s --usage [commands_with_args]...", config.Name)
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

// UsageInfo represents parsed usage information from a script
type UsageInfo struct {
	Summary string
	Usage   string
	Help    string
	Args    []ArgSpec
	Rest    string
}

// ArgSpec represents an argument specification
type ArgSpec struct {
	Name      string
	Type      string // "positional", "short", "long"
	Required  bool
	HasValue  bool
	ValueName string
	Exclusive bool
}

// extractUsageFromFile reads a script file and extracts usage information
func extractUsageFromFile(path string) (*UsageInfo, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	info := &UsageInfo{}
	inCommentBlock := false
	
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		
		// Skip shebang
		if strings.HasPrefix(line, "#!") {
			continue
		}
		
		// Check if we're in a comment block
		if strings.HasPrefix(line, "#") {
			inCommentBlock = true
			line = strings.TrimSpace(line[1:]) // Remove #
			
			if strings.HasPrefix(line, "Summary:") {
				info.Summary = strings.TrimSpace(line[8:])
			} else if strings.HasPrefix(line, "Usage:") {
				info.Usage = strings.TrimSpace(line[6:])
				info.Args = parseUsageString(info.Usage)
			} else if line != "" {
				// Extended help text
				if info.Help != "" {
					info.Help += "\n"
				}
				info.Help += line
			}
		} else if inCommentBlock && line == "" {
			// Empty line continues comment block
			continue
		} else {
			// Non-comment line breaks the comment block
			break
		}
	}

	return info, scanner.Err()
}

// parseUsageString parses a usage string and extracts argument specifications
func parseUsageString(usage string) []ArgSpec {
	var args []ArgSpec
	
	// Split the usage line by spaces and process each token
	tokens := strings.Fields(usage)
	
	i := 0
	for i < len(tokens) {
		token := tokens[i]
		
		// Skip {cmd}
		if token == "{cmd}" {
			i++
			continue
		}
		
		if strings.HasPrefix(token, "<") && strings.HasSuffix(token, ">") {
			// Required positional: <name>
			name := token[1 : len(token)-1]
			args = append(args, ArgSpec{
				Name:     name,
				Type:     "positional",
				Required: true,
			})
		} else if strings.HasPrefix(token, "[") && strings.Contains(token, "]...") {
			// Rest args: [args]...
			inner := token[1:strings.Index(token, "]")]
			args = append(args, ArgSpec{
				Name: inner,
				Type: "rest",
			})
		} else if strings.HasPrefix(token, "[") && strings.HasSuffix(token, "]") {
			// Optional something: [...]
			inner := token[1 : len(token)-1]
			
			// Check if the next token is "..." to handle [args]...
			if i+1 < len(tokens) && tokens[i+1] == "..." {
				// Rest args: [args]...
				args = append(args, ArgSpec{
					Name: inner,
					Type: "rest",
				})
				// Skip the "..." token
				i++
			} else if strings.HasSuffix(inner, "...") {
				// Rest args: [args...] (alternative format)
				name := inner[:len(inner)-3]
				args = append(args, ArgSpec{
					Name: name,
					Type: "rest",
				})
			} else if strings.HasPrefix(inner, "-") && !strings.HasPrefix(inner, "--") {
				// Short flag: [-u]
				args = append(args, ArgSpec{
					Name:     inner,
					Type:     "short",
					Required: false,
				})
			} else if strings.HasPrefix(inner, "--") {
				if strings.Contains(inner, "=") {
					// Long flag with value: [--value=VALUE]
					parts := strings.SplitN(inner, "=", 2)
					args = append(args, ArgSpec{
						Name:      parts[0],
						Type:      "long",
						Required:  false,
						HasValue:  true,
						ValueName: parts[1],
					})
				} else {
					// Long flag: [--long]
					args = append(args, ArgSpec{
						Name:     inner,
						Type:     "long",
						Required: false,
					})
				}
			} else {
				// Optional positional: [name]
				args = append(args, ArgSpec{
					Name:     inner,
					Type:     "positional",
					Required: false,
				})
			}
		} else if strings.HasPrefix(token, "[") && strings.HasSuffix(token, "]!") {
			// Exclusive flag: [--exclusive]!
			inner := token[1 : len(token)-2] // Remove [ and ]!
			args = append(args, ArgSpec{
				Name:      inner,
				Type:      "long",
				Required:  false,
				Exclusive: true,
			})
		} else if strings.HasPrefix(token, "--") && strings.Contains(token, "=") {
			// Required long flag with value: --value=VALUE
			parts := strings.SplitN(token, "=", 2)
			args = append(args, ArgSpec{
				Name:      parts[0],
				Type:      "long",
				Required:  true,
				HasValue:  true,
				ValueName: parts[1],
			})
		}
		
		i++
	}
	
	return args
}

// parseArgsWithUsage parses command line arguments according to usage specification
func parseArgsWithUsage(args []string, specs []ArgSpec) (map[string]string, error) {
	result := make(map[string]string)
	
	// Initialize default values for flags
	for _, spec := range specs {
		if spec.Type == "short" || spec.Type == "long" {
			key := spec.Name
			if spec.Type == "short" {
				key = key[1:] // Remove -
			} else {
				key = key[2:] // Remove --
			}
			if !spec.HasValue {
				result[key] = "false"
			}
		}
	}
	
	// Get positional specs in order
	positionalSpecs := []ArgSpec{}
	var restSpec *ArgSpec
	var exclusiveUsed *ArgSpec
	
	for _, spec := range specs {
		if spec.Type == "positional" {
			positionalSpecs = append(positionalSpecs, spec)
		} else if spec.Type == "rest" {
			restSpec = &spec
		}
	}
	
	// Parse arguments
	i := 0
	positionalIndex := 0
	var restArgs []string
	
	for i < len(args) {
		arg := args[i]
		processed := false
		
		if strings.HasPrefix(arg, "--") {
			// Long flag
			parts := strings.SplitN(arg, "=", 2)
			flagName := parts[0]
			
			// Find matching spec
			for _, spec := range specs {
				if spec.Type == "long" && spec.Name == flagName {
					key := spec.Name[2:] // Remove --
					
					// Check for exclusive arguments
					if spec.Exclusive {
						if exclusiveUsed != nil {
							return nil, fmt.Errorf("exclusive argument %s cannot be used with other arguments", spec.Name)
						}
						exclusiveUsed = &spec
					}
					
					if spec.HasValue {
						if len(parts) > 1 {
							result[key] = parts[1]
						} else if i+1 < len(args) {
							result[key] = args[i+1]
							i++
						}
					} else {
						result[key] = "true"
					}
					processed = true
					break
				}
			}
		} else if strings.HasPrefix(arg, "-") && len(arg) == 2 {
			// Short flag
			for _, spec := range specs {
				if spec.Type == "short" && spec.Name == arg {
					key := spec.Name[1:] // Remove -
					
					// Check for exclusive arguments
					if spec.Exclusive {
						if exclusiveUsed != nil {
							return nil, fmt.Errorf("exclusive argument %s cannot be used with other arguments", spec.Name)
						}
						exclusiveUsed = &spec
					}
					
					result[key] = "true"
					processed = true
					break
				}
			}
		}
		
		if !processed {
			// Not a recognized flag, treat as positional or rest
			if positionalIndex < len(positionalSpecs) {
				spec := positionalSpecs[positionalIndex]
				result[spec.Name] = arg
				positionalIndex++
			} else if restSpec != nil {
				// Goes to rest args
				restArgs = append(restArgs, arg)
			} else {
				// No rest spec, but we have extra args - put them in rest anyway
				restArgs = append(restArgs, arg)
			}
		}
		i++
	}
	
	// Check if exclusive argument was used with other arguments
	if exclusiveUsed != nil && len(result) > 1 {
		// Count how many non-default values we have
		nonDefaultCount := 0
		for _, value := range result {
			if value != "false" && value != "" {
				nonDefaultCount++
			}
		}
		if nonDefaultCount > 1 {
			return nil, fmt.Errorf("exclusive argument %s cannot be used with other arguments", exclusiveUsed.Name)
		}
	}
	
	// Set rest args if we have them
	if len(restArgs) > 0 && restSpec != nil {
		result[restSpec.Name] = strings.Join(restArgs, " ")
	}
	
	// Validate required positional arguments
	for _, spec := range positionalSpecs {
		if spec.Required {
			if _, exists := result[spec.Name]; !exists {
				return nil, fmt.Errorf("missing required argument: %s", spec.Name)
			}
		}
	}
	
	return result, nil
}

func findSubcommand(config *Config, commandsWithArgs []string) (Command, error) {
	libexecPath := config.LibexecPath()
	
	// Check if libexec directory exists
	if info, err := os.Stat(libexecPath); err != nil || !info.IsDir() {
		return nil, SubError{Type: "", Message: fmt.Sprintf("%s: libexec directory not found in root", config.Name)}
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
			return nil, SubError{Type: "", Message: fmt.Sprintf("no such sub command '%s'", head)}
		}

		nextPath := filepath.Join(path, head)
		
		if _, err := os.Stat(nextPath); os.IsNotExist(err) {
			return nil, SubError{Type: "", Message: fmt.Sprintf("no such sub command '%s'", head)}
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
				return NewFileCommand(names, nextPath, remaining, config), nil
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
	readmePath := filepath.Join(d.path, "README")
	if info, err := extractUsageFromFile(readmePath); err == nil && info.Summary != "" {
		return info.Summary
	}
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
	
	help := d.Summary()
	if help != "Directory command" {
		help = help + "\n\n" + usage
	} else {
		help = usage
	}
	
	// Try to get extended help from README
	readmePath := filepath.Join(d.path, "README")
	if info, err := extractUsageFromFile(readmePath); err == nil && info.Help != "" {
		help += "\n\nArguments:\n  [commands_with_args]...\n\nOptions:\n  -h, --help  Print help\n\n" + info.Help
	}
	
	help += "\n\nAvailable subcommands:\n"
	for _, sub := range d.Subcommands() {
		summary := sub.Summary()
		if summary != "" && summary != "Directory command" {
			help += fmt.Sprintf("    %-12s %s\n", sub.Name(), summary)
		} else {
			help += fmt.Sprintf("    %s\n", sub.Name())
		}
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
					commands = append(commands, NewFileCommand(names, fullPath, []string{}, d.config))
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
	var errors []ValidationError
	
	// Validate all subcommands recursively
	for _, sub := range d.Subcommands() {
		errors = append(errors, sub.Validate()...)
	}
	
	return errors
}

// FileCommand implements Command for executable files
type FileCommand struct {
	names     []string
	path      string
	args      []string
	config    *Config
	usageInfo *UsageInfo
}

func NewFileCommand(names []string, path string, args []string, config *Config) *FileCommand {
	usageInfo, _ := extractUsageFromFile(path)
	return &FileCommand{
		names:     names,
		path:      path,
		args:      args,
		config:    config,
		usageInfo: usageInfo,
	}
}

func (f *FileCommand) Name() string {
	return f.names[len(f.names)-1]
}

func (f *FileCommand) Summary() string {
	if f.usageInfo != nil {
		return f.usageInfo.Summary
	}
	return ""
}

func (f *FileCommand) Usage() (string, error) {
	if f.usageInfo != nil && f.usageInfo.Usage != "" {
		// Replace {cmd} with actual command
		cmdName := f.config.Name
		if len(f.names) > 0 {
			cmdName = f.config.Name + " " + strings.Join(f.names, " ")
		}
		usage := strings.Replace(f.usageInfo.Usage, "{cmd}", cmdName, -1)
		return fmt.Sprintf("Usage: %s", usage), nil
	}
	
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
	
	help := usage
	if f.usageInfo != nil && f.usageInfo.Help != "" {
		help += "\n\n" + f.usageInfo.Help
	}
	
	return help, nil
}

func (f *FileCommand) Subcommands() []Command {
	return []Command{}
}

func (f *FileCommand) Completions() (int, error) {
	// Check if the script supports completions by executing it with special environment
	cmd := exec.Command(f.path)
	env := os.Environ()
	env = append(env, fmt.Sprintf("_%s_COMPLETE", strings.ToUpper(f.config.Name))+"=true")
	if len(f.args) > 0 {
		env = append(env, fmt.Sprintf("_%s_COMPLETE_ARG", strings.ToUpper(f.config.Name))+"="+f.args[0])
	}
	cmd.Env = env
	
	output, err := cmd.Output()
	if err != nil {
		return 0, nil // No completions available
	}
	
	fmt.Print(string(output))
	return 0, nil
}

func (f *FileCommand) Invoke() (int, error) {
	// Set environment variables
	envName := fmt.Sprintf("_%s_ROOT", strings.ToUpper(f.config.Name))
	os.Setenv(envName, f.config.Root)
	
	// Set XDG cache directory environment variable
	cacheEnvName := fmt.Sprintf("_%s_CACHE", strings.ToUpper(f.config.Name))
	homeDir, err := os.UserHomeDir()
	if err == nil {
		cacheDir := filepath.Join(homeDir, ".cache", f.config.Name, "cache")
		os.Setenv(cacheEnvName, cacheDir)
	}
	
	// Parse arguments according to usage and set environment variable
	argsEnvName := fmt.Sprintf("_%s_ARGS", strings.ToUpper(f.config.Name))
	if f.usageInfo != nil && len(f.usageInfo.Args) > 0 {
		// Only do strict validation if we have specific usage args defined
		hasSpecificUsage := false
		for _, spec := range f.usageInfo.Args {
			if spec.Type != "rest" {
				hasSpecificUsage = true
				break
			}
		}
		
		if hasSpecificUsage {
			parsedArgs, err := parseArgsWithUsage(f.args, f.usageInfo.Args)
			if err != nil {
				return 1, err
			}
			
			// Format as key-value pairs for the environment variable in the order they appear in the usage spec
			var argPairs []string
			
			// Process arguments in the order they appear in the usage specification
			for _, spec := range f.usageInfo.Args {
				var key string
				switch spec.Type {
				case "positional", "rest":
					key = spec.Name
				case "short":
					key = spec.Name[1:] // Remove -
				case "long":
					key = spec.Name[2:] // Remove --
				}
				
				if value, exists := parsedArgs[key]; exists {
					argPairs = append(argPairs, fmt.Sprintf(`%s "%s"`, key, value))
				}
			}
			
			os.Setenv(argsEnvName, strings.Join(argPairs, " "))
		} else {
			// Simple usage with just rest args, pass through
			os.Setenv(argsEnvName, strings.Join(f.args, " "))
		}
	} else {
		// No usage info, just pass raw arguments
		os.Setenv(argsEnvName, strings.Join(f.args, " "))
	}
	
	// Execute the script
	cmd := exec.Command(f.path, f.args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	
	if err := cmd.Run(); err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			return exitError.ExitCode(), nil
		}
		// Format error message to match expected format
		if strings.Contains(err.Error(), "exec format error") {
			return 1, fmt.Errorf("%s: Exec format error (os error 8)", f.config.Name)
		}
		if strings.Contains(err.Error(), "no such file or directory") {
			return 1, fmt.Errorf("%s: No such file or directory (os error 2)", f.config.Name)
		}
		return 1, fmt.Errorf("%s: %v", f.config.Name, err)
	}
	
	return 0, nil
}

func (f *FileCommand) Validate() []ValidationError {
	var errors []ValidationError
	
	// Validate usage string if present
	if f.usageInfo != nil && f.usageInfo.Usage != "" {
		usage := f.usageInfo.Usage
		
		// Check for basic malformed patterns
		if strings.Contains(usage, "[") && !strings.Contains(usage, "]") {
			errors = append(errors, ValidationError{
				Path:    f.path,
				Message: "malformed usage string: unmatched brackets",
			})
		}
		if strings.Contains(usage, "<") && !strings.Contains(usage, ">") {
			errors = append(errors, ValidationError{
				Path:    f.path,
				Message: "malformed usage string: unmatched angle brackets", 
			})
		}
	}
	
	return errors
}

func handleError(config *Config, err error, silent bool) {
	if !silent {
		if subErr, ok := err.(SubError); ok {
			fmt.Fprintf(os.Stderr, "%s\n", subErr.Message)
		} else {
			// Don't add prefix if error already starts with config.Name
			errMsg := err.Error()
			if strings.HasPrefix(errMsg, config.Name+":") {
				fmt.Fprintf(os.Stderr, "%s\n", errMsg)
			} else {
				fmt.Fprintf(os.Stderr, "%s: %v\n", config.Name, err)
			}
		}
	}
	os.Exit(1)
}