# Match making algorithms modeling 
Modeling a set of the matchmaking alogirthms, providing statistics and data related to simulation.

Right now supporting:
- Random choise matchamaking 
- FIFO matchmaking
- Skill level search matchmaking (greedy algorithm)
- Real skill level generation (Uniform and Normal distribution)
- Matchmaking skill level update (Elo)

# Usage and parameters 
1. Run executable, the report would be generated in reports folder
2. Use view.html in the main project folder to load\analyse generated report
3. Check the executable help (-h flag) for the list of possible options 
4. ???
5. PROFIT!!!

# Command line arguments 
```
USAGE:
    mmmodel [FLAGS] [OPTIONS]

FLAGS:
    -h, --help              Prints help information
        --use_real_skill    Always use real skill level as skill level of the user
    -V, --version           Prints version information

OPTIONS:
    -a, --alg <algorithm>                                Algorithm type [default: rnd]  [values: fifo, rnd, skill]
        --continuous_play_prob <continuous_play_prob>
            The probability that after a game user will join the queue [default: 0.0] 
        --max_game_length <max_game_length>              The amount of time before user reenter queue [default: 300] 
    -n <name>                                            Name of the simulation
        --prefill_factor <prefill_factor>
            Amount of users to be added to the team on the first run of the search algorithm [default: 0.0] 
        --queue_factor <queue_factor>                    Queue overloading factor [default: 1.0] 
        --rmax <real_skill_max>                          Maximum value of the skill level [default: 2200] 
        --rmin <real_skill_min>                          Minimum value of the skill level [default: 800] 
    -d, --search_delay <search_delay>                    Delay between searches in ticks [default: 10] 
    -s <skill>                                           Default skill level assigned to the user [default: 1500] 
        --team_size <team_size>                          The size of the team [default: 5] 
    -t <time>                                            A period of time to simulate in seconds [default: 86400] 
    -u <users_at_start>                                  Amount of users to be generated [default: 500] 
    -g <users_to_gen>                                    Amount of users to be generated [default: 500] 
```
