(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.Map
     * @extends alchemy.core.Oculus
     */
    alchemy.formula.add({
        name: 'arena.Map',
        extend: 'alchemy.core.Oculus',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.ApplicationModule'
        }],
        overrides: {
            /** @lends arena.Map */

            init: function hocuspocus(_super) {
                return function () {
                    this.observe(this.messages, 'app:start', this.initMap, this);

                    _super.call(this);
                };
            },

            prepare: function () {
                this.resources.define({
                    id: 'mapdata',
                    src: 'data/maps.json'
                });
            },

            initMap: function () {
                var map = this.resources.get('mapdata').maps[0];

                this.tiles = alchemy('Collectum').brew();
                for (var i = 0; i < map.tiles.length; i++) {
                    var row = map.tiles[i];
                    for (var j = 0; j < row.length; j++) {
                        var cfg = map.tileTypes[row.charAt(j)];
                        if (cfg) {
                            this.tiles.add(alchemy.mix({
                                id: this.createKey(j, i),
                                row: i,
                                col: j
                            }, cfg));
                        }
                    }
                }
                this.startPos = map.startPos;

                this.mapView = this.viewFactory.createView(this, {
                    map: this,
                    target: '#map',
                    messages: this.messages
                });

                this.messages.trigger('map:init', {
                    map: this,
                    view: this.mapView
                });
            },

            createKey: function (col, row) {
                return row + '#' + col;
            },

            getStartPos: function () {
                return this.startPos;
            },

            getTile: function (col, row) {
                return this.tiles.get(this.createKey(col, row));
            },

            isBlocked: function (col,  row) {
                var tile = this.getTile(col, row);
                return tile && tile.type !== 'floor';
            },

            // function A*(start,goal)
            //      closedset := the empty set    // The set of nodes already evaluated.
            //      openset := {start}    // The set of tentative nodes to be evaluated, initially containing the start node
            //      came_from := the empty map    // The map of navigated nodes.
            //
            //      g_score[start] := 0    // Cost from start along best known path.
            //      // Estimated total cost from start to goal through y.
            //      f_score[start] := g_score[start] + heuristic_cost_estimate(start, goal)
            //
            //      while openset is not empty
            //          current := the node in openset having the lowest f_score[] value
            //          if current = goal
            //              return reconstruct_path(came_from, goal)
            //
            //          remove current from openset
            //          add current to closedset
            //          for each neighbor in neighbor_nodes(current)
            //              tentative_g_score := g_score[current] + dist_between(current,neighbor)
            //              if neighbor in closedset
            //                  if tentative_g_score >= g_score[neighbor]
            //                      continue
            //
            //              if neighbor not in openset or tentative_g_score < g_score[neighbor]
            //                  came_from[neighbor] := current
            //                  g_score[neighbor] := tentative_g_score
            //                  f_score[neighbor] := g_score[neighbor] + heuristic_cost_estimate(neighbor, goal)
            //                  if neighbor not in openset
            //                      add neighbor to openset
            //
            //      return failure
            //
            //  function reconstruct_path(came_from, current_node)
            //      if current_node in came_from
            //          p := reconstruct_path(came_from, came_from[current_node])
            //          return (p + current_node)
            //      else
            //          return current_node

            getPath: function (startCol, startRow, endCol, endRow) {
                var openList;
                var closedList;
                var current;
                var nodes;

                if (this.isBlocked(startCol, startCol)) {
                    return null;
                }

                if (this.isBlocked(endCol, endRow)) {
                    return null;
                }

                openList = [{
                    col: startCol,
                    row: startRow,
                    startDisance: 0,
                    path: []
                }];
                closedList = {};
                nodes = {};

                while (openList.length > 0) {
                    current = openList.pop();
                    if (current.col === endCol && current.row === endRow) {
                        return current.path;
                    }
                    closedList[this.createKey(current.col, current.row)] = current;

                    alchemy.each(this.collectNeighbors(current), function (neighbor) {
                        var n = closedList[this.createKey(neighbor.col, neighbor.row)];
                        if (n && n.startDisance < neighbor.startDisance) {
                            return;
                        }
                        // TODO: implement remaining parts using the improved Collectum


                    }, this);

                }
                return null;
            },

            collectNeighbors: function (curr) {
                var neighbors = [];
                alchemy.each([[0, -1], [1, -1], [1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0], [-1, -1]], function (offset) {
                    var neighborCol = curr.col + offset[0];
                    var neighborRow = curr.row + offset[0];

                    if (!this.isBlocked(neighborCol, neighborRow)) {
                        neighbors.push({
                            col: neighborCol,
                            row: neighborRow,
                            startDisance: curr.startDisance + 1,
                            path: curr.path.concat(curr)
                        });
                    }
                }, this);
                return neighbors;
            },

            estimateDistance: function (x1, y1, x2, y2) {
                var dx = x2 - x1;
                var dy = y2 - y1;
                return Math.max(Math.abs(dx), Math.abs(dy));
            }
        }
    });
}());

