#!/usr/bin/env python3
"""Synthetic overlay host. No game start, profile access, persistence or screen capture."""
import argparse
import json
from pathlib import Path
import queue
import subprocess
import threading
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--helper', type=Path, required=True)
    parser.add_argument('--geometry', type=Path, required=True, help='compiled overlay_window_macos.swift')
    parser.add_argument('--pid', type=int, default=0, help='game PID; 0 previews over the desktop')
    parser.add_argument('--preview-unfocused', action='store_true', help='visual testing only: show even if another app is focused')
    parser.add_argument('--seconds', type=int, default=20)
    args = parser.parse_args()
    def geometry():
        return json.loads(subprocess.check_output([str(args.geometry.resolve()), str(args.pid)], text=True))
    baseline = geometry()['front_pid']
    output = queue.Queue()
    helper = subprocess.Popen([str(args.helper.resolve()), '--prototype'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    def lines():
        for line in helper.stdout:
            output.put(line.rstrip())
    threading.Thread(target=lines, daemon=True).start()
    try:
        ready = json.loads(output.get(timeout=20))
        if ready.get('type') != 'ready':
            raise RuntimeError(f'not ready: {ready}')
        print(json.dumps({'ready':ready,'helper_pid':helper.pid}), flush=True)
        end = time.monotonic() + args.seconds
        next_notice = 0
        seq = 0
        sent = 0
        focus_changes = 0
        samples = []
        showing = False
        visible_until = 0
        while time.monotonic() < end and helper.poll() is None:
            state = geometry()
            # Never display above other apps while testing a game-owned notification.
            if state['found'] and (args.pid == 0 or args.preview_unfocused or state['front_pid'] == args.pid) and time.monotonic() >= next_notice:
                seq += 1
                sent += 1
                message = {'v':1,'seq':seq,'id':[21,7,1,0,35][(sent-1)%5],'title':'CUSA03173','x':round(state['x']+state['width']-480),'y':round(state['y']+24),'duration_ms':4000}
                helper.stdin.write(json.dumps(message)+'\n')
                helper.stdin.flush()
                next_notice = time.monotonic() + 8
                showing = True
                visible_until = time.monotonic() + 4
                print(json.dumps({'sent':seq,'window':state}), flush=True)
            if showing and args.pid and (not state['found'] or (not args.preview_unfocused and state['front_pid'] != args.pid)):
                seq += 1
                helper.stdin.write(json.dumps({'v':1,'seq':seq,'id':7,'x':0,'y':0,'duration_ms':1000,'visible':False})+'\n')
                helper.stdin.flush()
                showing = False
                visible_until = 0
            if state['front_pid'] == helper.pid:
                focus_changes += 1
            stats = subprocess.check_output(['ps','-o','pcpu=,rss=','-p',str(helper.pid)],text=True).split()
            if len(stats)==2:
                samples.append([float(stats[0]),int(stats[1]), "visible" if time.monotonic()<visible_until else "idle"])
            while not output.empty():
                print(output.get_nowait(),flush=True)
            time.sleep(1)
        print(json.dumps({'notices':sent, 'last_sequence':seq,'helper_became_frontmost_samples':focus_changes,'starting_front_pid':baseline,
                          'helper_main_cpu_mean':sum(x[0] for x in samples)/len(samples) if samples else None,
                          'helper_main_cpu_by_phase':{phase:sum(x[0] for x in samples if x[2]==phase)/max(1,sum(x[2]==phase for x in samples)) for phase in ('visible','idle')},
                          'helper_main_rss_max_kib':max((x[1] for x in samples),default=None),
                          'note':'main helper process only; excludes WebKit processes'}),flush=True)
    finally:
        helper.stdin.close()
        try:
            code = helper.wait(timeout=5)
        except subprocess.TimeoutExpired:
            helper.terminate()
            code = helper.wait(timeout=5)
        print(json.dumps({'helper_exit':code}),flush=True)


if __name__ == '__main__':
    main()
