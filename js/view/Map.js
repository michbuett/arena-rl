/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.view.Map
     * @extends arena.view.Prima
     */
    alchemy.formula.add({
        name: 'arena.view.Map',
        extend: 'arena.view.Prima',
        overrides: {
            tileWidth: 32,
            tileHeight: 32,
            tileTemplate: [
                '<div',
                ' class="tile <$=data.type $>"',
                ' style="left:<$=data.x$>px; top:<$=data.y$>px;"',
                ' data-column="<$=data.col$>"',
                ' data-row="<$=data.row$>"',
                '></div>'
            ].join(''),

            init: function hocuspocus(_super) {
                return function () {
                    this.on('rendered', function () {
                        $('#map .tile').on('click', this.tileClick.bind(this));
                    }, this);
                    _super.call(this);
                };
            },

            tileClick: function (ev) {
                var target = ev && ev.target;
                if (target) {
                    this.trigger('tile:click', $(ev.target).data());
                }
            },

            render: function (ctxt) {
                alchemy.each(this.map.tiles, function (tile, i, ctxt) {
                    ctxt.push(alchemy.render(this.tileTemplate, alchemy.mix({
                        x: this.tileWidth * tile.col,
                        y: this.tileHeight * tile.row
                    }, tile)));
                }, this, [ctxt]);
                return ctxt;
            },

            update: function () {}
        }
    });
}());

