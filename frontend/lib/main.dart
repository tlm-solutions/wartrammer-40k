import 'package:flutter/material.dart';
import 'dart:async';
import 'dart:convert';
import 'dart:core';
import 'package:universal_io/io.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'WarTrammer 40k',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MyHomePage(title: 'WarTrammer 40k'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  Stopwatch watch = Stopwatch();
  Timer timer = Timer(Duration(milliseconds: 100), () {});
  String elapsedTime = 'Not running';
  var client = HttpClient();
  String host = Uri.base.origin.toString(); // 'http://localhost:8000'
  String line = '';
  String run = '';

  getWatchTimeString() {
    return (watch.elapsed.inHours % 24).toString().padLeft(2, "0") +
        ":" +
        (watch.elapsed.inMinutes % 60).toString().padLeft(2, "0") +
        ":" +
        (watch.elapsed.inSeconds % 60).toString().padLeft(2, "0");
  }

  void updateTime(Timer timer) {
    if (watch.isRunning) {
      setState(() {
        elapsedTime = "Running for " + getWatchTimeString();
      });
    }
  }

  httpErrorDialog(String error) => showDialog(
        context: context,
        builder: (BuildContext context) => AlertDialog(
          title: const Text('HTTP Error'),
          content: Text(error),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Close'),
            ),
          ],
        ),
      );

  callApiGet(String endpoint, callback) async {
    try {
      print("Calling endpoint $host$endpoint");
      var request = await client.getUrl(Uri.parse(host + endpoint));
      var response = await request.close();
      if (response.statusCode != 200) {
        throw Exception("HTTP status code is " +
            response.statusCode.toString() +
            " not 200.");
      }
      final stringData = await response.transform(utf8.decoder).join();
      var decoded = json.decode(stringData);
      if (decoded['success'] == true) {
        callback();
      } else {
        throw Exception("Error occured in backend: $decoded");
      }
    } catch (e) {
      print("Error: " + e.toString());
      httpErrorDialog(e.toString());
    }
  }

  callApiPost(String endpoint, body, callback) async {
    try {
      print("Calling endpoint $host$endpoint");
      var request = await client.postUrl(Uri.parse(host + endpoint));
      request.headers.contentType =
          ContentType('application', 'json', charset: 'utf-8');
      request.headers.contentLength = body.length;
      request.write(body);
      var response = await request.close();
      if (response.statusCode != 200) {
        throw Exception("HTTP status code is " +
            response.statusCode.toString() +
            " not 200.");
      }
      final stringData = await response.transform(utf8.decoder).join();
      var decoded = json.decode(stringData);
      if (decoded['success'] == true) {
        callback();
      } else {
        throw Exception("Error occured in backend: $decoded");
      }
    } catch (e) {
      print("Error: " + e.toString());
      httpErrorDialog(e.toString());
    }
  }

  // TODO: fetch current state (started, stopped, free to start at reload) and write to watch
  // GET /api/state
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: PreferredSize(
        preferredSize: const Size.fromHeight(90),
        child: AppBar(
          title: Text(widget.title, style: TextStyle(fontSize: 25.0)),
          bottom: PreferredSize(
            preferredSize: const Size.fromHeight(40),
            child: Container(
              margin: EdgeInsets.only(left: 15, bottom: 10, right: 15, top: 0),
              child: Align(
                  alignment: Alignment.centerLeft,
                  child: Text(elapsedTime,
                      style: TextStyle(fontSize: 25.0, color: Colors.white))),
            ),
          ),
        ),
      ),
      body: Padding(
        padding: EdgeInsets.only(top: 25),
        child: Column(
          children: <Widget>[
            Row(
              children: <Widget>[
                Spacer(),
                Expanded(
                  flex: 10,
                  child: TextField(
                    decoration: InputDecoration(
                      border: OutlineInputBorder(),
                      labelText: 'Line number',
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (value) => line = value,
                  ),
                ),
                Spacer(),
                Expanded(
                  flex: 10,
                  child: TextField(
                    decoration: InputDecoration(
                      border: OutlineInputBorder(),
                      labelText: 'Run number',
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (value) => run = value,
                  ),
                ),
                Spacer(),
              ],
            ),
            DataTable(
              columns: const <DataColumn>[
                DataColumn(
                  label: Expanded(
                    child: Text(
                      'Start time',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  ),
                ),
                DataColumn(
                  label: Expanded(
                    child: Text(
                      'Duration',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  ),
                ),
                DataColumn(
                  label: Expanded(
                    child: Text(
                      'Line number',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  ),
                ),
                DataColumn(
                  label: Expanded(
                    child: Text(
                      'Run number',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  ),
                ),
              ],
              rows: <DataRow>[
                DataRow(
                  cells: <DataCell>[
                    DataCell(Text('22:07')),
                    DataCell(Text('01:10')),
                    DataCell(Text('3')),
                    DataCell(Text('8')),
                  ],
                ),
                DataRow(
                  cells: <DataCell>[
                    DataCell(Text('18:02')),
                    DataCell(Text('00:23')),
                    DataCell(Text('5')),
                    DataCell(Text('20')),
                  ],
                ),
                DataRow(
                  cells: <DataCell>[
                    DataCell(Text('17:40')),
                    DataCell(Text('00:20')),
                    DataCell(Text('1')),
                    DataCell(Text('2')),
                  ],
                ),
              ],
            ),
          ],
        ),
      ),
      floatingActionButton: Wrap(
        direction: Axis.vertical,
        children: <Widget>[
          Container(
            margin: EdgeInsets.all(10),
            child: FloatingActionButton(
                backgroundColor: watch.isRunning ? Colors.red : Colors.green,
                onPressed: () async {
                  if (watch.isRunning) {
                    callApiGet(
                        '/api/stop',
                        () => setState(() {
                              watch.stop();
                              elapsedTime =
                                  "Stopped at " + getWatchTimeString();
                            }));
                  } else {
                    callApiGet(
                        '/api/start',
                        () => setState(() {
                              watch.reset();
                              watch.start();
                              timer = Timer.periodic(
                                  Duration(milliseconds: 100), updateTime);
                            }));
                  }
                },
                child: watch.isRunning ? Icon(Icons.stop) : Icon(Icons.start)),
          ),
          Container(
            margin: EdgeInsets.all(10),
            child: FloatingActionButton(
                backgroundColor: watch.elapsedTicks > 0 && !watch.isRunning
                    ? Colors.blue
                    : Colors.grey,
                onPressed: watch.elapsedTicks > 0 && !watch.isRunning
                    ? () => {
                          callApiPost(
                              '/api/line_info',
                              json.encode({"line": line, "run": run}),
                              () => callApiGet(
                                  '/api/finish',
                                  () => setState(() {
                                        watch.stop();
                                        watch.reset();
                                        elapsedTime = "Not running";
                                      })))
                        }
                    : null,
                child: Icon(Icons.check)),
          ),
        ],
      ),
    );
  }
}
