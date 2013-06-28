/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.view.Map
     * @extends alchemy.browser.View
     */
    alchemy.formula.add({
        name: 'arena.view.Map',
        extend: 'alchemy.browser.View',
        overrides: {
            tileWidth: 32,
            tileHeight: 32,
            tileTemplate: [
                '<div',
                ' class="tile <$=data.type $>"',
                ' style="left:<$=data.x$>px; top:<$=data.y$>px;"',
                ' data-column="<$=data.col$>"',
                ' data-row="<$=data.row$>" >',
                '  <div class="bg"></div>',
                '  <div class="fg"></div>',
                '</div>'
            ].join(''),

            init: function hocuspocus(_super) {
                return function () {
                    this.tiles = [];
                    this.map.tiles.each(function (tile) {
                        this.tiles.push(alchemy.mix({
                            x: this.getScreenX(tile.col),
                            y: this.getScreenY(tile.row)
                        }, tile));
                    }, this);


                    // initialize mouse events
                    this.on('rendered', function () {
                        this.boundMouseDownHandler = this.mouseDownHandler.bind(this);
                        this.boundMouseUpHandler = this.mouseUpHandler.bind(this);
                        this.boundMouseMoveHandler = this.mouseMoveHandler.bind(this);
                        this.boundClickHandler = this.clickHandler.bind(this);

                        $('#map').on('mousedown', this.boundMouseDownHandler);
                        $('#map').on('mouseup', this.boundMouseUpHandler);
                        $('#map').on('mousemove', this.boundMouseMoveHandler);
                        $('#map').on('click', this.boundClickHandler);
                    }, this);

                    _super.call(this);
                };
            },

            getScreenX: function (mapX) {
                return this.tileWidth * mapX;
            },

            getScreenY: function (mapY) {
                return this.tileHeight * mapY;
            },

            getMapX: function (screenX) {
                return screenX / this.tileWidth;
            },

            getMapY: function (screenY) {
                return screenY / this.tileHeight;
            },


            render: function (ctxt) {
                alchemy.each(this.tiles, function (tile, i, ctxt) {
                    ctxt.push(alchemy.render(this.tileTemplate, tile));
                }, this, [ctxt]);
                return ctxt;
            },

            dispose: function hocuspocus(_super) {
                return function () {
                    $('#map').off('mousedown', this.boundMouseDownHandler);
                    $('#map').off('mouseup', this.boundMouseUpHandler);
                    $('#map').off('mousemove', this.boundMouseMoveHandler);
                    $('#map').off('click', this.boundClickHandler);

                    _super.call(this);
                };
            },

            //
            //
            // private helper
            //
            //

            mouseDownHandler: function (e) {
                this.trigger('map:mousedown', this.getEventPosition(e));
            },

            mouseUpHandler: function (e) {
                this.trigger('map:mouseup', this.getEventPosition(e));
            },

            mouseMoveHandler: function (e) {
                this.trigger('map:hover', this.getEventPosition(e));
            },

            clickHandler: function (e) {
                this.trigger('map:click', this.getEventPosition(e));

                var tile = e && e.target && $(e.target).parent('.tile');
                if (tile && tile.length > 0) {
                    this.trigger('tile:click', alchemy.mix(e, tile.data()));
                }
            },

            getEventPosition: function (e) {
                return alchemy.mix(e, {
                    mapX: this.getMapX(e.pageX),
                    mapY: this.getMapY(e.pageY),
                    screenX: e.pageX,
                    screenY: e.pageY
                });
            }
        }
    });
}());

