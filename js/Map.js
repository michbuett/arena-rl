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
            /** @lends arena.Map.prototype */

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
                return !tile || tile.type !== 'floor';
            },


            /**
             * Callculates the shortes path from the civen start tile to a goal tile using
             * an A* algorithm
             *
             * @param {Object} start The start point
             * @param {Number} start.col The column index of the start point
             * @param {Number} start.row The row index of the start point
             *
             * @param {Object} goal The goal point
             * @param {Number} goal.col The column index of the goal point
             * @param {Number} goal.row The row index of the goal point
             *
             * @return {Object[]} An array of map points the leads from the start
             *      to the goal. Each object provides teh properties <code>col</code>
             *      and <code>row</code>; It returns <code>null</code> if there is no
             *      path
             */
            getPath: (function () {
                var neighborOffsets = [[0, -1], [1, -1], [1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0], [-1, -1]];

                // helper to find all available neighbor tiles
                function collectNeighbors(nodeList, curr, map) {
                    var neighbors = [];
                    alchemy.each(neighborOffsets, function (offset) {
                        var neighborCol = curr.col + offset[0];
                        var neighborRow = curr.row + offset[1];
                        var key = this.createKey(neighborCol, neighborRow);

                        if (!this.isBlocked(neighborCol, neighborRow)) {
                            if (!nodeList.contains(key)) {
                                var startDistance = curr.startDistance + 1;

                                nodeList.add({
                                    id: key,
                                    col: neighborCol,
                                    row: neighborRow,
                                    startDistance: startDistance
                                });
                            }

                            neighbors.push(nodeList.get(key));
                        }
                    }, map);

                    return neighbors;
                }

                // helper to create the path array for the found result
                function createPath(node) {
                    var path = [{
                        col: node.col,
                        row: node.row
                    }];
                    node = node.cameFrom;

                    while (node) {
                        path.unshift({
                            col: node.col,
                            row: node.row
                        });
                        node = node.cameFrom;
                    }
                    return path;
                }

                // helper method that explores a single neighbor tile
                function processNeighbor(n, index, current, goal, openList, closedList, map) {
                    var newStartDistance = current.startDistance + map.distance(current.col, current.row, n.col, n.row);
                    if (closedList.contains(n) && n.startDistance <= newStartDistance) {
                        return;
                    }
                    if (!openList.contains(n) || n.startDistance > newStartDistance) {
                        n.cameFrom = current;
                        n.startDistance = newStartDistance;
                        n.goalDistance = newStartDistance + map.distance(n.col, n.row, goal.col, goal.row);
                        openList.add(n);
                    }
                }

                return function (start, goal) {
                    if (!alchemy.isObject(start) || this.isBlocked(start.col, start.row)) {
                        // no valid start point -> exit
                        return null;
                    }

                    if (!alchemy.isObject(goal) || this.isBlocked(goal.col, goal.row)) {
                        // no valid end point -> exit
                        return null;
                    }

                    var nodes = alchemy('Collectum').brew();
                    var openList = alchemy('Collectum').brew({
                        next: function () {
                            var min = this.at(0);
                            for (var i = 1, l = this.items.length; i < l; i++) {
                                var item = this.items[i];
                                if (item.goalDistance < min.goalDistance) {
                                    min = item;
                                }
                            }
                            return min;
                        }
                    });
                    var closedList = alchemy('Collectum').brew();
                    var current;
                    var run = 0;

                    nodes.add({
                        id: this.createKey(start.col, start.row),
                        col: start.col,
                        row: start.row,
                        startDistance: 0,
                        goalDistance: this.distance(start.col, start.row, goal.col, goal.row)
                    });
                    openList.add(nodes.at(0));

                    while (openList.length > 0) {
                        if (run > 1000) {
                            // Prevent endless loop while developing
                            // TODO: remove this after exhaustive testing
                            window.console.warn('To many iterations! Possible endless loop!');
                            return null;
                        }

                        // get the item with smallest distance to the goal from the open list
                        current = openList.next();
                        if (current.col === goal.col && current.row === goal.row) {
                            // the current candidat is the goal tile
                            // -> return result
                            return createPath(current);
                        } else {
                            // the current candidat is not the goal
                            // -> mark it as "processed"
                            closedList.add(current);
                            openList.remove(current);
                            // -> ... and look its at its neighbors
                            var neighbors = collectNeighbors(nodes, current, this);
                            alchemy.each(neighbors, processNeighbor, this, [current, goal, openList, closedList, this]);
                        }
                        run++;
                    }
                    // there is no path
                    return null;
                };
            }()),

            /**
             * Estimates the distance between two map points (tiles) A and B ignoring possible obstacles
             * (based on http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#heuristics-for-grid-maps)
             *
             * @param {Number} x The x-coordinate of point A
             * @param {Number} y The y-coordinate of point A
             * @param {Number} x The x-coordinate of point B
             * @param {Number} y The y-coordinate of point B
             * @return {Number} The estimated distance
             */
            distance: (function () {
                var Dp = 1; // distance for perpendicular (non-diaginal) steps
                var Dd = Math.sqrt(2); // distance for diaginal steps

                return function (x1, y1, x2, y2) {
                    var dx = Math.abs(Math.floor(x1) - Math.floor(x2));
                    var dy = Math.abs(Math.floor(y1) - Math.floor(y2));
                    return (Dp * (dx + dy) + (Dd - 2 * Dp) * Math.min(dx, dy));
                };
            }())
        }
    });
}());

