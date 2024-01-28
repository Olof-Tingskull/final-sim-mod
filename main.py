import os
import json
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np


class Data:
    road_lengths = []
    flow_rates = []
    max_flow_rates = []
    densities = []
    num_lanes = []
    stop_rates = []
    biases = []
    collisions = []
    steps = []
    movement = []

    def __init__(self, path):
        raw = []
        for filename in os.listdir(path):
            [name, extenstion] = filename.split('.')
            if extenstion == 'json':
                file_path = os.path.join(path, filename)
                with open(file_path, 'r') as f:
                    obj = json.load(f)
                    index = int(name.split('-')[1])
                    raw.append({**obj, **{"index": index}})

        raw.sort(key=lambda x: x["index"])

        for r in raw:
            config = r['config']
            result = r['result']

            if not isinstance(result, dict):
                continue

            if config["num_lanes"] == 0:
                continue

            if result["max_flow_rate"] == 0:
                continue

            self.flow_rates.append(result["flow_rate"])
            self.road_lengths.append(config["road_length"])
            self.max_flow_rates.append(result['max_flow_rate'])
            self.densities.append(config["car_density"])
            self.num_lanes.append(config["num_lanes"])
            self.biases.append(config["current_lane_bias"])
            self.stop_rates.append(config["random_stop_rate"])
            self.collisions.append(result["collisions"])
            self.steps.append(config["steps_to_run"])
            self.movement.append(config["max_movement"])
        
        self.flow_rates = np.array(self.flow_rates)
        self.road_lengths = np.array(self.road_lengths)
        self.max_flow_rates = np.array(self.max_flow_rates)
        self.densities = np.array(self.densities)
        self.num_lanes = np.array(self.num_lanes)
        self.biases = np.array(self.biases)
        self.stop_rates = np.array(self.stop_rates)
        self.collisions = np.array(self.collisions)
        self.movement = np.array(self.movement)
        self.steps = np.array(self.steps)


def plot_road_flow():
    d = Data("./results/road")
    
    plt.plot(d.road_lengths, d.flow_rates)
    plt.xlabel("Road Length")
    plt.xscale('log')
    plt.ylabel("Flow Rate")
    plt.title("Flow Rate vs Road Length")
    plt.show()

def plot_lanes_flow():
    d = Data("./results/lanes")
    
    plt.plot(d.num_lanes, d.flow_rates / d.num_lanes)
    plt.xlabel("Number of Lanes")
    plt.xscale('log')
    plt.ylabel("Flow Rate per Lane")
    plt.title("Flow Rate per Lane vs Number of Lanes")
    plt.show()


def plot_steps_flow():
    d = Data("./results/steps")
    
    plt.plot(d.steps, d.flow_rates)
    plt.xlabel("Simulation Duration (Steps)")
    plt.xscale('log')
    plt.ylabel("Flow Rate")
    plt.title("Flow Rate vs Simulation Duration")
    plt.show()

def plot_movement_flow():
    d = Data("./results/movement")
    
    plt.plot(d.movement, d.flow_rates / d.movement)
    plt.xlabel("Maximum Per-Step Movement")
    plt.xscale('log')
    plt.ylabel("Flow Rate per Velocity")
    plt.title("Flow Rate per Velocity vs Maximum Per-Step Movement")
    plt.show()
