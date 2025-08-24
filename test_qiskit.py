# test_qiskit.py
from qiskit import QuantumCircuit
from qiskit.providers.aer import AerSimulator
from qiskit import transpile, assemble, execute

qc = QuantumCircuit(2)
qc.h(0)
qc.cx(0, 1)
qc.measure_all()

sim = AerSimulator()

tqc = transpile(qc, sim)
qobj = assemble(tqc)
result = sim.run(qobj).result()
print(result.get_counts())
