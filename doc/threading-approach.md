# Threading Approach
Updating is not separated from rendering as this would complicate world state
synchronization to a point where it wouldn't be a benefit anymore. Instead, the
plan is to spawn worker tasks to offload work to different threads.
