import os
import json
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np

data = []
for filename in os.listdir('./result'):
    if filename.startswith('sim-') and filename.endswith('.json'):
        with open("./result/" + filename, 'r') as f:
            data.append(json.load(f))



flow_rates = []
max_flow_rates = []
utilizations = []
average_speeds = []
densities = []
num_lanes = []
stop_rates = []
biases = []

for d in data:
    config = d['config']
    result = d['result']

    if not isinstance(result, dict):
        continue

    if config["num_lanes"] == 0:
        continue

    if result["max_flow_rate"] == 0:
        continue

    flow_rates.append(result["flow_rate"])
    max_flow_rates.append(result['max_flow_rate'])
    utilizations.append(result["flow_rate"] / result['max_flow_rate'])
    densities.append(config["car_density"])
    num_lanes.append(config["num_lanes"])
    biases.append(config["current_lane_bias"])
    stop_rates.append(config["random_stop_rate"])
    average_speeds.append(result["flow_rate"] / config["car_density"] / config["num_lanes"])

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')



ax.scatter(biases, stop_rates, flow_rates)
#ax.scatter(bias, densities, np.log((max_flow_rates)))


ax.set_xlabel('Current Lane Bias')
ax.set_ylabel('Random Stop Rate')
ax.set_zlabel('Utilization')

plt.show()