# Processor

### Assumptions

1. Transactions occur **chronologically** in the input batch to the processor.
2. Transactions IDs are globally unique.

### Processor

Processor processes the transactions in batches. 
A batch can come from a file, part of a file, an HTTP request, etc.
The processor uses a *Reader* to read input, and a *Writer* to write output.
Processor processes the input in accordance to assumption [1]. 
Therefore, implementors of *Readers* must be aware of the order of the data that they return. 

If a batch contains invalid malformed record data, 
the batch is discarded before applying it to the state of the system.


### Possible optimizations
* If transaction IDs were unique per client and conveyed the chronological order of the data,
we could safely apply partially valid batches.




