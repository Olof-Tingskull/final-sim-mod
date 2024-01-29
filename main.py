import os
import json
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from scipy.interpolate import griddata
import numpy as np
plt.rcParams["font.family"] = "Times New Roman"


class Data:
    def __init__(self, path):
        self.road_lengths = []
        self.flow_rates = []
        self.max_flow_rates = []
        self.densities = []
        self.num_lanes = []
        self.stop_rates = []
        self.biases = []
        self.collisions = []
        self.steps = []
        self.movement = []
        self.acceleration = []
        self.braking = []

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
            self.acceleration.append(config["acceleration_rate"])
            self.braking.append(config["break_rate"])
        
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
        self.acceleration = np.array(self.acceleration)
        self.braking = np.array(self.braking)
        self.utilizations = self.flow_rates / self.max_flow_rates


def plot_road_flow():
    d = Data("./results/road")
    
    plt.plot(d.road_lengths, d.flow_rates)
    plt.xlabel("Road Length", fontsize=14)
    plt.xscale('log')
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/road.png")
    plt.show()

def plot_lanes_flow():
    d = Data("./results/lanes")
    
    plt.plot(d.num_lanes, (d.flow_rates / d.num_lanes))
    plt.xlabel("Number of Lanes", fontsize=14)
    plt.xscale('log')
    plt.ylabel("Flow Rate per Lane", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/lanes.png")
    plt.show()


def plot_steps_flow():
    d = Data("./results/steps")
    
    plt.plot(d.steps, d.flow_rates)
    plt.xlabel("Simulation Duration (Steps)", fontsize=14)
    plt.xscale('log')
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/steps.png")
    plt.show()

def plot_movement_flow():
    d = Data("./results/movement")
    
    plt.plot(d.movement, d.flow_rates / d.movement)
    plt.xlabel("Maximum Per-Step Movement", fontsize=14)
    plt.xscale('log')
    plt.ylabel("Flow Rate per Velocity", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/step_movement.png")
    plt.show()

def plot_3d_surface(
    xdata, ydata, zdata,
    xlabel, ylabel, zlabel,
    name
):
    fig = plt.figure()

    ax = fig.add_subplot(111, projection='3d')

    x = np.linspace(min(xdata), max(xdata), len(xdata))
    y = np.linspace(min(ydata), max(ydata), len(ydata))
    X, Y = np.meshgrid(x, y)

    Z = griddata((xdata, ydata), zdata, (X, Y), method='cubic')

    # Plot the surface
    ax.plot_surface(X, Y, Z)

    ax.set_xlabel(xlabel, fontsize=14)
    ax.set_ylabel(ylabel, fontsize=14)
    ax.set_zlabel(zlabel, fontsize=14)

    plt.tight_layout()
    plt.savefig(f"./report/{name}.png")
    plt.show()

def plot_density_bias_3d():
    d = Data("./results/density_bias_smaller")

    plot_3d_surface(
        d.densities,
        d.biases,
        d.flow_rates / d.max_flow_rates,
        "Car Density",
        "Current Lane Bias",
        "Utilization",
        "density_bias_3d"
    )

def averge_over_same(x, y):
    x_unique = np.unique(x)
    y_unique = np.zeros(len(x_unique))

    for i in range(len(x_unique)):
        x_val = x_unique[i]
        y_val = 0
        count = 0

        for j in range(len(x)):
            if x[j] == x_val:
                y_val += y[j]
                count += 1

        y_val /= count
        y_unique[i] = y_val

    return (x_unique, y_unique)


def plot_density_bias():
    d0025 = Data("./results/bias-d-0.025")
    d005 = Data("./results/bias-d-0.05")
    d01 = Data("./results/bias-d-0.1")
    d02 = Data("./results/bias-d-0.2")


    plt.plot(d0025.biases, d0025.flow_rates / d0025.max_flow_rates, label="Density = 0.025")
    plt.plot(d005.biases, d005.flow_rates / d005.max_flow_rates, label="Density = 0.05")
    plt.plot(d01.biases, d01.flow_rates / d01.max_flow_rates, label="Density = 0.1")
    plt.plot(d02.biases, d02.flow_rates / d02.max_flow_rates, label="Density = 0.2")
    plt.xlabel("Current Lane Bias", fontsize=14)
    plt.ylabel("Utilization", fontsize=14)
    plt.legend()
    plt.tight_layout()
    plt.savefig("./report/bias.png")
    plt.show()

def plot_stop_bias():
    d00005 = Data("./results/bias-sr-0.00005")
    d0001 = Data("./results/bias-sr-0.0001")
    d0002 = Data("./results/bias-sr-0.0002")
    d0004 = Data("./results/bias-sr-0.0004")
    d0008 = Data("./results/bias-sr-0.0008")
    d0016 = Data("./results/bias-sr-0.0016")

    plt.plot(d00005.biases, d00005.flow_rates, label="Stop Rate = 0.00005")
    plt.plot(d0001.biases, d0001.flow_rates, label="Stop Rate = 0.0001")
    plt.plot(d0002.biases, d0002.flow_rates, label="Stop Rate = 0.0002")
    plt.plot(d0004.biases, d0004.flow_rates, label="Stop Rate = 0.0004")
    plt.plot(d0008.biases, d0008.flow_rates, label="Stop Rate = 0.0008")
    plt.plot(d0016.biases, d0016.flow_rates, label="Stop Rate = 0.0016")
    plt.xlabel("Current Lane Bias", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.legend()
    plt.tight_layout()
    plt.savefig("./report/bias.png")
    plt.show()

def plot_methods_stop():
    bd = Data("./results/stop-bd")
    no = Data("./results/stop-no")
    fl = Data("./results/stop-fl")
    bl = Data("./results/stop-bl")

    plt.plot(no.stop_rates, no.flow_rates, label="No Lane Changing")
    plt.plot(bd.stop_rates, bd.flow_rates, label="Bi-directional")
    plt.plot(fl.stop_rates, fl.flow_rates, label="Forwad-looking")
    plt.plot(bl.stop_rates, bl.flow_rates, label="Backward-looking")
    
    plt.legend(fontsize=14)
    plt.xlabel("Spontaneous Braking Rate", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/methods-stop.png")
    plt.show()

def plot_methods_bias():
    bd = Data("./results/bias-bd")
    bl = Data("./results/bias-bl")
    fl = Data("./results/bias-fl")

    plt.plot(bd.biases, bd.flow_rates, label="Bi-directional")
    plt.plot(fl.biases, fl.flow_rates, label="Forwad-looking")
    plt.plot(bl.biases, bl.flow_rates, label="Backward-looking")
    
    plt.legend(fontsize=14)
    plt.xlabel("Current Lane Bias", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/methods-bias.png")
    plt.show()

def plot_methods_density():
    bd = Data("./results/density-bd")
    bl = Data("./results/density-bl")
    fl = Data("./results/density-fl")
    no = Data("./results/density-no")

    plt.plot(bd.densities, bd.flow_rates, label="Bi-directional")
    plt.plot(fl.densities, fl.flow_rates, label="Forwad-looking")
    plt.plot(bl.densities, bl.flow_rates, label="Backward-looking")
    plt.plot(no.densities, no.flow_rates, label="No Lane Changing")
    
    plt.legend(fontsize=14)
    plt.xlabel("Vehicle Density", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/methods-density.png")
    plt.show()

def plot_methods_acceleration():
    bd = Data("./results/acceleration-bd")
    bl = Data("./results/acceleration-bl")
    fl = Data("./results/acceleration-fl")
    no = Data("./results/acceleration-no")

    plt.plot(bd.acceleration, bd.flow_rates, label="Bi-directional")
    plt.plot(fl.acceleration, fl.flow_rates, label="Forwad-looking")
    plt.plot(bl.acceleration, bl.flow_rates, label="Backward-looking")
    plt.plot(no.acceleration, no.flow_rates, label="No Lane Changing")
    
    plt.legend(fontsize=14)
    plt.xlabel("Acceleration Rate", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/methods-acceleration.png")
    plt.show()

def plot_methods_braking():
    bd = Data("./results/braking-bd")
    bl = Data("./results/braking-bl")
    fl = Data("./results/braking-fl")
    no = Data("./results/braking-no")

    plt.plot(bd.braking, bd.flow_rates, label="Bi-directional")
    plt.plot(fl.braking, fl.flow_rates, label="Forwad-looking")
    plt.plot(bl.braking, bl.flow_rates, label="Backward-looking")
    plt.plot(no.braking, no.flow_rates, label="No Lane Changing")
    
    plt.legend(fontsize=14)
    plt.xlabel("Deceleration Rate", fontsize=14)
    plt.ylabel("Flow Rate", fontsize=14)
    plt.tight_layout()
    plt.savefig("./report/methods-braking.png")
    plt.show()


plot_methods_density()
