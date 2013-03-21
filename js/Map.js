(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.Map
     * @extends arena.View
     */
    alchemy.formula.add({
        name: 'arena.Map',
        extend: 'alchemy.core.Oculus',
        overrides: {
            tileTypes: {
                '.': {
                    type: 'floor'
                },
                '#': {
                    type: 'wall'
                },
                '~': {
                    type: 'water'
                }
            },

            map: [
                '      ###########      ',
                '    ###..#...#..###    ',
                '   ##....#...#....##   ',
                '  ##.....##.##.....##  ',
                ' ##.................## ',
                ' #...................# ',
                '##......#..~..#......##',
                '#..........~..........#',
                '#..........~..........#',
                '####.......~.......####',
                '#..#.......~.......#..#',
                '#.......~~~~~~~.......#',
                '#..#.......~.......#..#',
                '####.......~.......####',
                '#..........~..........#',
                '#..........~..........#',
                '##......#..~..#......##',
                ' #...................# ',
                ' ##.................## ',
                '  ##.....##.##.....##  ',
                '   ##....#...#....##   ',
                '    ###..#...#..###    ',
                '      ###########      '
            ],

            startPos: [11, 1],

            init: function hocuspocus(_super) {
                return function () {
                    this.tiles = [];
                    for (var i = 0; i < this.map.length; i++) {
                        var row = this.map[i];
                        for (var j = 0; j < row.length; j++) {
                            var cfg = this.tileTypes[row.charAt(j)];
                            if (cfg) {
                                this.tiles.push(alchemy.mix({
                                    row: i,
                                    col: j
                                }, cfg));
                            }
                        }
                    }

                    _super.call(this);
                };
            },

            getStartPos: function () {
                return this.startPos;
            }
        }
    });
}());

