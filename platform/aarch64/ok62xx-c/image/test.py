
# 读取gic.txt文件

# 一行一行的读取，若这一行中gic_handle_irq，则提取"xxx"的值
# 将这个值放入元组中

import os


# 读取gic.txt文件
def read_gic_file(file_path):
    gic_list = []
    with open(file_path, 'r') as file:
        for line in file:
            if 'gic_handle_irq' in line:
                # 提取"xxx"的值
                start_index = line.find('"') + 1
                end_index = line.find('"', start_index)
                gic_value = line[start_index:end_index]
                if gic_value not in gic_list:
                    # 将值放入元组中
                    gic_list.append(gic_value)
    return gic_list

def main():
    # 获取当前脚本所在目录
    current_dir = os.path.dirname(os.path.abspath(__file__))
    # 拼接gic.txt的路径
    gic_file_path = os.path.join(current_dir, 'gic.txt')
    
    # 读取gic.txt文件
    gic_list = read_gic_file(gic_file_path)
    
    # 打印结果
    if not gic_list:
        print("没有找到gic_handle_irq的值")
    else:
        print("提取的gic_handle_irq值：")
        # gic_list进行排序
        gic_list.sort()
        for gic in gic_list:
            print(gic)
if __name__ == "__main__":
    main()


